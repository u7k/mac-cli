//! Descriptor-relative storage: a renamed directory or symlink cannot redirect a journal write.
use crate::error::{Error, Result};
#[cfg(unix)]
use std::os::{
    fd::{AsRawFd, FromRawFd},
    unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
};
use std::{
    ffi::CString,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};
const LIMIT: u64 = 1024 * 1024;

pub fn read_bounded(path: &Path) -> Result<Vec<u8>> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NONBLOCK);
    bounded(options.open(path)?)
}
fn bounded(file: File) -> Result<Vec<u8>> {
    if !file.metadata()?.is_file() {
        return Err(Error::invalid("Expected a regular file."));
    }
    let mut bytes = Vec::new();
    file.take(LIMIT + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > LIMIT {
        return Err(Error::new(
            "input_too_large",
            "Configuration files must not exceed 1 MiB.",
        ));
    }
    Ok(bytes)
}
pub struct State {
    directory: File,
    _lock: File,
}
impl State {
    pub fn open(root: &Path) -> Result<Self> {
        fs::create_dir_all(root)?;
        let directory = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(root)?;
        let metadata = directory.metadata()?;
        if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o022 != 0 {
            return Err(Error::new(
                "unsafe_state",
                "The state directory must be owned by you and not writable by other users.",
            ));
        }
        directory.set_permissions(fs::Permissions::from_mode(0o700))?;
        let lock = open_at(
            &directory,
            "lock",
            libc::O_CREAT | libc::O_RDWR | libc::O_NONBLOCK,
        )?;
        check_private(&lock)?;
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(Error::new("busy", "Another settings operation is running."));
        }
        Ok(Self {
            directory,
            _lock: lock,
        })
    }
    pub fn read(&self, name: &str) -> Result<Vec<u8>> {
        let file = open_at(&self.directory, name, libc::O_RDONLY | libc::O_NONBLOCK)?;
        check_private(&file)?;
        bounded(file)
    }
    pub fn write(&self, name: &str, bytes: &[u8]) -> Result<()> {
        let destination = component(name)?;
        let temporary = format!(
            ".journal-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        let temp = component(&temporary)?;
        let mut file = open_at(
            &self.directory,
            &temporary,
            libc::O_CREAT | libc::O_EXCL | libc::O_WRONLY,
        )?;
        let result = (|| -> Result<()> {
            file.write_all(bytes)?;
            file.sync_all()?;
            if unsafe {
                libc::renameat(
                    self.directory.as_raw_fd(),
                    temp.as_ptr(),
                    self.directory.as_raw_fd(),
                    destination.as_ptr(),
                )
            } != 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
            self.directory.sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            unsafe {
                libc::unlinkat(self.directory.as_raw_fd(), temp.as_ptr(), 0);
            }
        }
        result
    }
}
fn component(name: &str) -> Result<CString> {
    if name.is_empty() || name == "." || name == ".." || name.contains('/') {
        return Err(Error::invalid("Invalid state file name."));
    }
    CString::new(name).map_err(|_| Error::invalid("Invalid state file name."))
}
fn open_at(directory: &File, name: &str, flags: i32) -> Result<File> {
    let name = component(name)?;
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            flags | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}
fn check_private(file: &File) -> Result<()> {
    let m = file.metadata()?;
    if !m.is_file()
        || m.uid() != unsafe { libc::geteuid() }
        || m.nlink() != 1
        || m.mode() & 0o077 != 0
    {
        return Err(Error::new(
            "unsafe_state",
            "State files must be private, singly-linked regular files owned by you.",
        ));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_symlink_directory_without_chmod() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");
        fs::create_dir(&target).unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).unwrap();
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(State::open(&link).is_err());
        assert_eq!(fs::metadata(target).unwrap().mode() & 0o777, 0o755);
    }
    #[test]
    fn anchored_directory_survives_parent_rename() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("state");
        let state = State::open(&root).unwrap();
        let original = dir.path().join("original");
        fs::rename(&root, &original).unwrap();
        fs::create_dir(&root).unwrap();
        state.write("x.json", b"safe").unwrap();
        assert_eq!(fs::read(original.join("x.json")).unwrap(), b"safe");
        assert!(!root.join("x.json").exists());
    }
    #[test]
    fn rejects_journal_symlinks_and_nonprivate_files() {
        let dir = tempfile::tempdir().unwrap();
        let state = State::open(dir.path()).unwrap();
        let other = dir.path().join("other");
        fs::write(&other, b"{}").unwrap();
        std::os::unix::fs::symlink(other, dir.path().join("x.json")).unwrap();
        assert!(state.read("x.json").is_err());
        state.write("private.json", b"{}").unwrap();
        fs::set_permissions(
            dir.path().join("private.json"),
            fs::Permissions::from_mode(0o644),
        )
        .unwrap();
        assert!(state.read("private.json").is_err());
    }
    #[test]
    fn rejects_large_config_and_fifo() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("large");
        fs::write(&path, vec![b'a'; LIMIT as usize + 1]).unwrap();
        assert_eq!(read_bounded(&path).unwrap_err().code, "input_too_large");
        let fifo = dir.path().join("fifo");
        let c = CString::new(fifo.to_str().unwrap()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(c.as_ptr(), 0o600) }, 0);
        assert!(read_bounded(&fifo).is_err());
    }
}
