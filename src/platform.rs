pub fn validate(os: &str, arch: &str, major: Option<u32>) -> Result<(), &'static str> {
    if os != "macos" {
        return Err("mac-cli requires Apple macOS.");
    }
    if arch != "aarch64" {
        return Err("mac-cli requires Apple Silicon (arm64).");
    }
    match major {
        Some(v) if v >= 14 => Ok(()),
        Some(_) => Err("mac-cli requires macOS 14 or later."),
        None => Err("mac-cli could not determine the macOS version."),
    }
}

pub fn check() -> Result<(), &'static str> {
    #[cfg(not(target_os = "macos"))]
    {
        validate(std::env::consts::OS, std::env::consts::ARCH, None)
    }
    #[cfg(target_os = "macos")]
    {
        if std::env::consts::ARCH != "aarch64" {
            return validate("macos", std::env::consts::ARCH, None);
        }
        // Read the OS product version directly; no subprocess, settings, or feature bridge.
        let mut bytes = [0u8; 128];
        let mut size = bytes.len();
        let rc = unsafe {
            libc::sysctlbyname(
                c"kern.osproductversion".as_ptr(),
                bytes.as_mut_ptr().cast(),
                &mut size,
                std::ptr::null_mut(),
                0,
            )
        };
        let major = if rc == 0 {
            std::str::from_utf8(&bytes[..size.min(bytes.len())])
                .ok()
                .and_then(|s| s.trim_end_matches('\0').split('.').next()?.parse().ok())
        } else {
            None
        };
        validate("macos", "aarch64", major)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn platform_matrix() {
        for os in ["linux", "windows", "freebsd", "ios"] {
            assert_eq!(
                validate(os, "aarch64", Some(26)),
                Err("mac-cli requires Apple macOS.")
            );
        }
        assert!(validate("macos", "x86_64", Some(26))
            .unwrap_err()
            .contains("Apple Silicon"));
        assert!(validate("macos", "aarch64", Some(13)).is_err());
        assert!(validate("macos", "aarch64", None).is_err());
        for v in [14, 15, 26] {
            assert!(validate("macos", "aarch64", Some(v)).is_ok());
        }
    }
}
