use std::process::{Command, Output};
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mac"))
        .args(args)
        .env("LANG", "tr_TR.UTF-8")
        .env("LC_ALL", "tr_TR.UTF-8")
        .env("NO_COLOR", "1")
        .output()
        .unwrap()
}
#[cfg(not(target_os = "macos"))]
#[test]
fn rejects_non_macos_before_parsing_or_writing() {
    let dir = tempfile::tempdir().unwrap();
    for args in [
        vec!["--help"],
        vec!["--json", "settings", "set", "dock.autohide", "true"],
        vec!["not-a-command"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_mac"))
            .args(args)
            .env("HOME", dir.path())
            .env("XDG_STATE_HOME", dir.path())
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(78));
        assert!(result.stdout.is_empty());
        assert_eq!(
            String::from_utf8(result.stderr).unwrap(),
            "error: mac-cli requires Apple macOS.\n"
        );
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn english_help_even_in_a_turkish_environment() {
    let result = run(&["--help"]);
    assert!(result.status.success());
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.contains("Usage: mac"));
    assert!(text.contains("English-only"));
    assert!(!text.contains("Kullanım"));
    assert!(!text.contains('\u{1b}'));
}
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn every_registered_command_has_help() {
    let output = run(&["commands", "--json"]);
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["ok"], true);
    for item in envelope["data"].as_array().unwrap() {
        let parts = item["command"]
            .as_str()
            .unwrap()
            .split(' ')
            .skip(1)
            .chain(std::iter::once("--help"))
            .collect::<Vec<_>>();
        let result = run(&parts);
        assert!(
            result.status.success(),
            "{}: {}",
            item["command"],
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(String::from_utf8_lossy(&result.stdout).contains("Usage:"));
    }
}
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn noninteractive_confirmation_prevents_deletion() {
    let result = run(&["reminders", "delete", "not-an-id", "--json"]);
    assert!(!result.status.success());
    let envelope: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(envelope["error"]["code"], "confirmation_required");
    assert!(result.stderr.is_empty());
}
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn invalid_settings_do_not_create_state() {
    let dir = tempfile::tempdir().unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_mac"))
        .args(["settings", "set", "dock.size", "999", "--json"])
        .env("XDG_STATE_HOME", dir.path())
        .output()
        .unwrap();
    assert!(!result.status.success());
    let envelope: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(envelope["error"]["code"], "invalid_input");
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
}
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn experimental_catalog() {
    let normal: serde_json::Value =
        serde_json::from_slice(&run(&["settings", "list", "--json"]).stdout).unwrap();
    let all: serde_json::Value =
        serde_json::from_slice(&run(&["settings", "list", "--experimental", "--json"]).stdout)
            .unwrap();
    assert!(all["data"].as_array().unwrap().len() > normal["data"].as_array().unwrap().len());
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn native_preference_existence_is_a_boolean() {
    use std::io::Write;
    let mut process = Command::new(env!("CARGO_BIN_EXE_mac"))
        .arg("__bridge")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    process.stdin.take().unwrap().write_all(br#"{"op":"preferences.read","args":{"domain":"io.uygur.mac-cli.nonexistent-test","key":"missing"}}"#).unwrap();
    let result = process.wait_with_output().unwrap();
    let response: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(response["data"]["exists"].is_boolean(), "{response}");
}
