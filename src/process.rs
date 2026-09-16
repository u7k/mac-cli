use crate::error::{Error, Result};
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const OUTPUT_LIMIT: usize = 16 * 1024 * 1024;
fn collect(mut pipe: impl Read) -> std::io::Result<(Vec<u8>, bool)> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 8192];
    let mut overflow = false;
    loop {
        let n = pipe.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        let keep = n.min(OUTPUT_LIMIT.saturating_sub(bytes.len()));
        bytes.extend_from_slice(&chunk[..keep]);
        overflow |= keep < n;
    }
    Ok((bytes, overflow))
}
pub fn run(
    program: &str,
    args: &[String],
    input: Option<&[u8]>,
    timeout: Duration,
) -> Result<String> {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .env("LC_ALL", "C")
        .env("LANG", "en_US.UTF-8")
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd.spawn().map_err(|e| {
        Error::new(
            "process_start_failed",
            format!("Could not start {program}: {e}"),
        )
    })?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let out = std::thread::spawn(move || collect(stdout));
    let err = std::thread::spawn(move || collect(stderr));
    let writer = input.map(|bytes| {
        let bytes = bytes.to_vec();
        let mut pipe = child.stdin.take().unwrap();
        std::thread::spawn(move || pipe.write_all(&bytes))
    });
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break Some(status);
        }
        if start.elapsed() >= timeout {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(15));
    };
    // A grandchild may detach and retain a pipe. Never join a blocked I/O thread
    // indefinitely, and never signal a process group after its leader was reaped.
    let pipe_deadline =
        Instant::now() + Duration::from_secs(1).min(timeout.saturating_sub(start.elapsed()));
    while !out.is_finished()
        || !err.is_finished()
        || writer.as_ref().is_some_and(|w| !w.is_finished())
    {
        if Instant::now() >= pipe_deadline {
            return Err(Error::new(
                "timeout",
                "A subprocess kept an input or output stream open after its deadline.",
            ));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let (stdout, overflow) = out
        .join()
        .map_err(|_| Error::new("io_error", "Output reader failed."))??;
    let (stderr, err_overflow) = err
        .join()
        .map_err(|_| Error::new("io_error", "Error reader failed."))??;
    if let Some(writer) = writer {
        let _ = writer.join();
    }
    let Some(status) = status else {
        return Err(Error::new(
            "timeout",
            format!("{program} timed out after {} seconds.", timeout.as_secs()),
        ));
    };
    if !status.success() {
        let diagnostic = String::from_utf8_lossy(&stderr);
        let diagnostic = diagnostic.trim().chars().take(1500).collect::<String>();
        return Err(Error::new(
            "command_failed",
            format!(
                "{program} failed (exit {}).{}",
                status.code().unwrap_or(-1),
                if diagnostic.is_empty() {
                    String::new()
                } else {
                    format!(" {diagnostic}")
                }
            ),
        ));
    }
    if overflow || err_overflow {
        return Err(Error::new(
            "output_too_large",
            "The command output exceeded 16 MiB. Narrow the request.",
        ));
    }
    String::from_utf8(stdout)
        .map_err(|_| Error::new("invalid_response", "The command did not return UTF-8 text."))
}
pub fn tool(program: &str, args: &[&str]) -> Result<String> {
    run(
        program,
        &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        None,
        Duration::from_secs(30),
    )
}
pub fn elevated(program: &str, args: &[String]) -> Result<String> {
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() {
        return Err(Error::new(
            "authentication_required",
            "Run this command in an interactive terminal for administrator authentication.",
        ));
    }
    let status = Command::new("/usr/bin/sudo").arg("-v").status()?;
    if !status.success() {
        return Err(Error::new(
            "authentication_required",
            "Administrator authentication was not completed.",
        ));
    }
    let mut all = vec!["-n".into(), program.into()];
    all.extend_from_slice(args);
    run("/usr/bin/sudo", &all, None, Duration::from_secs(120))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn argument_boundaries_are_preserved() {
        let s = tool(
            "/usr/bin/printf",
            &["%s", "$(touch /tmp/mac-cli-injection); 'Unicode: Ç'"],
        )
        .unwrap();
        assert_eq!(s, "$(touch /tmp/mac-cli-injection); 'Unicode: Ç'");
    }
    #[test]
    fn timeout_and_errors_are_distinct() {
        assert_eq!(
            run("/bin/sleep", &["5".into()], None, Duration::from_millis(50))
                .unwrap_err()
                .code,
            "timeout"
        );
        assert_eq!(
            tool("/usr/bin/false", &[]).unwrap_err().code,
            "command_failed"
        );
    }
    #[test]
    fn stdin_is_delivered() {
        assert_eq!(
            run(
                "/bin/cat",
                &[],
                Some("hello\n世界".as_bytes()),
                Duration::from_secs(1)
            )
            .unwrap(),
            "hello\n世界"
        );
    }
}
