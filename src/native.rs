use crate::error::{Error, Result};
use serde_json::Value;
#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn mac_call(request: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    fn mac_free(response: *mut std::ffi::c_char);
}

fn invoke(op: &str, args: Value) -> Result<Value> {
    validate_request(op, &args)?;
    #[cfg(target_os = "macos")]
    {
        let request = std::ffi::CString::new(serde_json::to_string(
            &serde_json::json!({"op":op,"args":args}),
        )?)
        .map_err(|_| Error::invalid("Input contains a NUL byte."))?;
        let raw = unsafe { mac_call(request.as_ptr()) };
        if raw.is_null() {
            return Err(Error::new(
                "native_error",
                "The macOS bridge returned no response.",
            ));
        }
        let bytes = unsafe { std::ffi::CStr::from_ptr(raw).to_bytes().to_vec() };
        unsafe {
            mac_free(raw);
        }
        let value: Value = serde_json::from_slice(&bytes)?;
        if let Some(e) = value.get("error") {
            return Err(Error::new(
                e["code"].as_str().unwrap_or("native_error"),
                e["message"]
                    .as_str()
                    .unwrap_or("The macOS operation failed."),
            ));
        }
        Ok(value["data"].clone())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (op, args);
        Err(Error::new("unsupported", "mac-cli requires Apple macOS."))
    }
}
fn validate_request(op: &str, args: &Value) -> Result<()> {
    let object = args
        .as_object()
        .ok_or_else(|| Error::invalid("Native arguments must be an object."))?;
    // Uppercase denotes required fields; lowercase denotes optional fields.
    // S/s string, N/n unsigned device ID, F/f percentage, B/b boolean, V any JSON scalar.
    let fields: &[(&str, char)] = match op {
        "media.play" | "media.pause" | "battery" | "doctor" | "display.list" | "apps.list"
        | "session.lock" | "clipboard.read" | "clipboard.clear" | "network.ip" | "wifi.status"
        | "wifi.on" | "wifi.off" | "wifi.scan" | "bluetooth.status" | "bluetooth.on"
        | "bluetooth.off" | "bluetooth.scan" | "bluetooth.list" | "calendar.list"
        | "calendar.today" | "reminders.lists" => &[],
        "brightness" | "keyboard.brightness" | "audio.level" => &[
            ("action", 's'),
            ("value", 'f'),
            ("input", 'b'),
            ("display", 'n'),
        ],
        "display.modes" => &[("id", 'N')],
        "display.set" => &[("id", 'N'), ("mode", 'N')],
        "audio.list" => &[("input", 'b')],
        "audio.select" => &[("input", 'b'), ("id", 'N')],
        "keyboard.source" => &[("id", 's')],
        "wifi.connect" => &[("ssid", 'S'), ("password", 's')],
        "bluetooth.connect" | "bluetooth.disconnect" => &[("address", 'S')],
        "preferences.read" => &[("domain", 'S'), ("key", 'S')],
        "preferences.write" => &[("domain", 'S'), ("key", 'S'), ("value", 'V')],
        "calendar.events" => &[("start", 'S'), ("end", 'S')],
        "calendar.add" => &[
            ("title", 'S'),
            ("start", 'S'),
            ("end", 'S'),
            ("calendar", 's'),
        ],
        "calendar.delete" => &[("id", 'S'), ("start", 's')],
        "reminders.add" => &[("title", 'S'), ("list", 's')],
        "reminders.complete" | "reminders.delete" => &[("id", 'S')],
        "reminders.list" => &[("list", 's')],
        "reminders.search" => &[("query", 'S')],
        "apps.open" => &[("name", 'S')],
        "wallpaper" | "ocr" | "clipboard.image" => &[("path", 'S')],
        "clipboard.write" | "plist.decode" => &[("text", 'S')],
        _ => return Err(Error::invalid("Unknown native operation.")),
    };
    if object
        .keys()
        .any(|key| !fields.iter().any(|(name, _)| *name == key))
    {
        return Err(Error::invalid("Unknown native argument."));
    }
    for (name, kind) in fields {
        let Some(value) = object.get(*name) else {
            if kind.is_ascii_uppercase() {
                return Err(Error::invalid(format!("Missing native argument: {name}.")));
            }
            continue;
        };
        let valid = match kind.to_ascii_lowercase() {
            's' => value.as_str().is_some_and(|s| !s.contains('\0')),
            'n' => value
                .as_u64()
                .is_some_and(|n| n > 0 && n <= u32::MAX as u64),
            'f' => value
                .as_f64()
                .is_some_and(|n| n.is_finite() && (0.0..=100.0).contains(&n)),
            'b' => value.is_boolean(),
            'v' => value.is_null() || value.is_boolean() || value.is_number() || value.is_string(),
            _ => false,
        };
        if !valid {
            return Err(Error::invalid(format!("Invalid native argument: {name}.")));
        }
    }
    if ["brightness", "keyboard.brightness", "audio.level"].contains(&op) {
        let action = args["action"].as_str().unwrap_or("get");
        let allowed = if op == "audio.level" {
            &["get", "set", "up", "down", "mute", "unmute", "toggle"][..]
        } else {
            &["get", "set", "up", "down"][..]
        };
        if !allowed.contains(&action)
            || (["set", "up", "down"].contains(&action) && !object.contains_key("value"))
        {
            return Err(Error::invalid("Invalid native level action."));
        }
    }
    if op == "preferences.write" {
        crate::settings::validate_native_preference(
            args["domain"].as_str().unwrap(),
            args["key"].as_str().unwrap(),
            &args["value"],
        )?;
    }
    Ok(())
}

pub fn call(op: &str, args: Value) -> Result<Value> {
    let result = isolated(op, &args)?;
    // Confirm that a keyboard adjustment survives the short-lived native client.
    if op == "keyboard.brightness" && args["action"].as_str().is_some_and(|a| a != "get") {
        let after = isolated(op, &serde_json::json!({"action":"get"}))?;
        let expected = result["percent"].as_f64();
        let actual = after["percent"].as_f64();
        if expected.zip(actual).is_none_or(|(a, b)| (a - b).abs() > 2.) {
            return Err(Error::new("verification_failed","Keyboard brightness did not persist after the native client exited. This device needs a different backlight backend."));
        }
        return Ok(after);
    }
    Ok(result)
}
fn isolated(op: &str, args: &Value) -> Result<Value> {
    let executable = std::env::current_exe()?;
    let request = serde_json::to_vec(&serde_json::json!({"op":op,"args":args}))?;
    let output = crate::process::run(
        executable
            .to_str()
            .ok_or_else(|| Error::new("io_error", "The executable path is not UTF-8."))?,
        &["__bridge".into()],
        Some(&request),
        std::time::Duration::from_secs(75),
    )?;
    let response: Value = serde_json::from_str(output.trim())?;
    if let Some(e) = response.get("error") {
        return Err(Error::new(
            e["code"].as_str().unwrap_or("native_error"),
            e["message"]
                .as_str()
                .unwrap_or("The macOS operation failed."),
        ));
    }
    Ok(response["data"].clone())
}
pub fn worker() -> i32 {
    use std::io::{Read, Write};
    let result = (|| -> Result<Value> {
        let mut input = String::new();
        std::io::stdin()
            .take(16 * 1024 * 1024 + 1)
            .read_to_string(&mut input)?;
        if input.len() > 16 * 1024 * 1024 {
            return Err(Error::invalid("Native input exceeds 16 MiB."));
        }
        let request: Value = serde_json::from_str(&input)?;
        invoke(
            request["op"]
                .as_str()
                .ok_or_else(|| Error::invalid("Missing native operation."))?,
            request["args"].clone(),
        )
    })();
    let response = match result {
        Ok(data) => serde_json::json!({"data":data}),
        Err(error) => serde_json::json!({"error":error}),
    };
    let _ = writeln!(std::io::stdout(), "{response}");
    0
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn rejects_malformed_worker_requests_before_ffi() {
        for (op, args) in [
            ("media.toggle", json!({})),
            ("media.play", json!({"command":2})),
            ("media.pause", json!({"app":"Music"})),
            ("brightness", json!({"action":"set","value":200})),
            ("brightness", json!({"action":"set"})),
            ("brightness", json!({"action":"erase"})),
            ("audio.select", json!({"id":-1})),
            ("wifi.connect", json!({"ssid":null})),
            ("apps.open", json!({"name":"bad\u{0000}app"})),
            (
                "preferences.write",
                json!({"domain":"com.apple.finder","key":"unregistered","value":true}),
            ),
            ("battery", json!([])),
        ] {
            assert!(validate_request(op, &args).is_err(), "{op}");
        }
        assert!(validate_request(
            "keyboard.brightness",
            &json!({"action":"set","value":37.5,"input":false})
        )
        .is_ok());
        assert!(validate_request(
            "preferences.write",
            &json!({"domain":"com.apple.dock","key":"tilesize","value":null})
        )
        .is_ok());
    }
}
