use crate::{
    applications, cli,
    error::{Error, Result},
    native, output, process,
    settings::{self, Backend},
};
use clap::ArgMatches;
use serde_json::{json, Map, Value};
use std::{
    fs,
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

pub fn run() -> i32 {
    if std::env::args().nth(1).as_deref() == Some("__bridge") {
        return native::worker();
    }
    let matches = cli::command().get_matches();
    let mut path = Vec::new();
    let mut leaf = &matches;
    while let Some((name, next)) = leaf.subcommand() {
        path.push(name.to_string());
        leaf = next;
    }
    if path.first().map(String::as_str) == Some("menu") {
        if matches.get_flag("json") {
            return output::emit(
                "menu",
                Err(Error::invalid("The interactive menu cannot emit JSON.")),
                true,
                false,
            );
        }
        return output::emit("menu", menu(), false, matches.get_flag("plain"));
    }
    let result = execute(&path, leaf);
    output::emit(
        &if path.is_empty() {
            "status".into()
        } else {
            path.join(" ")
        },
        result,
        matches.get_flag("json"),
        matches.get_flag("plain"),
    )
}
fn text<'a>(m: &'a ArgMatches, name: &str) -> &'a str {
    m.try_get_one::<String>(name)
        .ok()
        .flatten()
        .map(String::as_str)
        .unwrap_or("")
}
fn strings(m: &ArgMatches, name: &str) -> Vec<String> {
    m.get_many::<String>(name)
        .map(|v| v.cloned().collect())
        .unwrap_or_default()
}
fn number(m: &ArgMatches, name: &str) -> u32 {
    *m.get_one::<u32>(name).unwrap_or(&0)
}
fn result_text(text: String) -> Result<Value> {
    Ok(json!({"text":text.trim_end()}))
}
fn confirm(m: &ArgMatches, description: &str) -> Result<()> {
    if m.get_flag("yes") {
        return Ok(());
    }
    if !io::stdin().is_terminal() {
        return Err(Error::new(
            "confirmation_required",
            format!("{description} Run interactively or supply --yes."),
        ));
    }
    if dialoguer::Confirm::new()
        .with_prompt(output::prompt_text(description))
        .default(false)
        .interact()
        .map_err(|e| Error::new("terminal_error", e.to_string()))?
    {
        Ok(())
    } else {
        Err(Error::new("cancelled", "The operation was cancelled."))
    }
}
fn values(m: &ArgMatches, fields: &[&str]) -> Value {
    let mut result = Map::new();
    for field in fields {
        if let Some(v) = m.try_get_one::<String>(field).ok().flatten() {
            result.insert(field.to_string(), json!(v));
        }
    }
    Value::Object(result)
}
fn level(op: &str, path: &[String], m: &ArgMatches, input: bool) -> Result<Value> {
    let action = path
        .last()
        .map(String::as_str)
        .filter(|s| ["get", "set", "up", "down", "mute", "unmute", "toggle"].contains(s))
        .unwrap_or_else(|| {
            if m.try_get_one::<u8>("percent").ok().flatten().is_some() {
                "set"
            } else {
                "get"
            }
        });
    let value = if action == "set" {
        m.get_one::<u8>("percent").copied().unwrap_or(0)
    } else if action == "up" || action == "down" {
        m.get_one::<u8>("step").copied().unwrap_or(5)
    } else {
        0
    };
    let mut args = json!({"action":action,"value":value,"input":input});
    if op == "brightness" {
        if let Some(display) = m.get_one::<u32>("display") {
            args["display"] = json!(display);
        }
    }
    native::call(op, args)
}
fn summary() -> Value {
    let mut result = Map::new();
    for (name, op, args) in [
        ("battery", "battery", json!({})),
        ("display", "brightness", json!({"action":"get"})),
        ("volume", "audio.level", json!({"action":"get"})),
    ] {
        result.insert(
            name.into(),
            match native::call(op, args) {
                Ok(v) => v,
                Err(e) => json!({"status":"Unavailable","reason":e.message}),
            },
        );
    }
    result.insert(
        "help".into(),
        json!("Run mac menu, mac commands, or mac <group> --help."),
    );
    Value::Object(result)
}
pub fn execute(path: &[String], m: &ArgMatches) -> Result<Value> {
    let p = path.iter().map(String::as_str).collect::<Vec<_>>();
    let exp = m.get_flag("experimental");
    match p.as_slice() {
        [] | ["status"] => Ok(summary()),
        ["doctor"] => native::call("doctor", json!({})),
        ["commands"] => {
            let rows=cli::leaves(&cli::command(),vec![]).into_iter().map(|(path,c)| {
                let family=path.first().cloned().unwrap_or_default();
                let requirement=match family.as_str() {
                    "brightness"|"display"=>"Compatible display; private brightness API",
                    "keyboard"=>"Backlit keyboard or selectable input source",
                    "calendar"=>"Calendar permission",
                    "reminders"=>"Reminders permission",
                    "play"|"pause"=>"An app registered with macOS Now Playing; private MediaRemote API",
                    "notes"|"music"=>"Application Automation permission",
                    "bluetooth"=>"Bluetooth hardware and permission",
                    "wifi"=>"Wi-Fi hardware; Location Services for network names",
                    "capture"=>"Screen Recording permission for screenshots",
                    _=>"macOS 14+ on Apple Silicon",
                };
                json!({"command":format!("mac {}",path.join(" ")),"description":c.get_about().map(ToString::to_string),"requirements":requirement,"support":"Device-dependent; run mac doctor"})
            }).collect::<Vec<_>>();
            Ok(json!(rows))
        }
        ["battery"] => native::call("battery", json!({})),
        ["brightness", ..] => level("brightness", path, m, false),
        ["keyboard", "brightness", ..] => level("keyboard.brightness", path, m, false),
        ["volume", ..] => level("audio.level", path, m, false),
        ["audio", "gain", ..] => level("audio.level", path, m, true),
        ["audio", direction, action] => {
            let mut a = json!({"input":*direction=="input"});
            if *action == "set" {
                a["id"] = json!(number(m, "id"));
            }
            native::call(
                if *action == "set" {
                    "audio.select"
                } else {
                    "audio.list"
                },
                a,
            )
        }
        ["display", "list"] => native::call("display.list", json!({})),
        ["display", "modes"] => native::call("display.modes", json!({"id":number(m,"id")})),
        ["display", "set"] => native::call(
            "display.set",
            json!({"id":number(m,"id"),"mode":number(m,"mode")}),
        ),
        ["keyboard", "source", "list"] => native::call("keyboard.source", json!({})),
        ["keyboard", "source", "set"] => native::call("keyboard.source", values(m, &["id"])),
        ["wifi", action] => {
            let mut a = json!({});
            if *action == "connect" {
                a["ssid"] = json!(text(m, "ssid"));
                if !m.get_flag("open") {
                    if !io::stdin().is_terminal() {
                        return Err(Error::new("input_required","Wi-Fi passwords must be entered in an interactive terminal. Use --open only for an open network."));
                    }
                    let password = dialoguer::Password::new()
                        .with_prompt("Wi-Fi password")
                        .interact()
                        .map_err(|e| Error::new("terminal_error", e.to_string()))?;
                    a["password"] = json!(password);
                }
            }
            native::call(&format!("wifi.{action}"), a)
        }
        ["bluetooth", action] => {
            native::call(&format!("bluetooth.{action}"), values(m, &["address"]))
        }
        ["network", "ip"] | ["network", "interfaces"] => native::call("network.ip", json!({})),
        ["network", "ports"] => {
            match process::tool("/usr/sbin/lsof", &["-nP", "-iTCP", "-sTCP:LISTEN"]) {
                Ok(t) => result_text(t),
                Err(e) if e.message.contains("exit 1") => {
                    Ok(json!({"text":"No listening TCP ports were found."}))
                }
                Err(e) => Err(e),
            }
        }
        ["network", "dns", action] => dns(action, m),
        ["power", "status"] | ["power", "low-power", "get"] => {
            result_text(process::tool("/usr/bin/pmset", &["-g", "custom"])?)
        }
        ["power", "low-power", state] => {
            let value = if *state == "on" { "1" } else { "0" };
            process::elevated(
                "/usr/bin/pmset",
                &["-a".into(), "lowpowermode".into(), value.into()],
            )?;
            result_text(process::tool("/usr/bin/pmset", &["-g", "custom"])?)
        }
        ["power", "sleep-after"] => {
            let timer = if text(m, "target") == "display" {
                "displaysleep"
            } else {
                "sleep"
            };
            process::elevated(
                "/usr/bin/pmset",
                &["-a".into(), timer.into(), number(m, "minutes").to_string()],
            )?;
            result_text(process::tool("/usr/bin/pmset", &["-g", "custom"])?)
        }
        ["power", "awake"] => {
            let seconds = number(m, "seconds");
            // Inherit the terminal so Ctrl-C ends the assertion immediately.
            let status = std::process::Command::new("/usr/bin/caffeinate")
                .args(["-i", "-t", &seconds.to_string()])
                .status()?;
            if !status.success() {
                return Err(Error::new(
                    "cancelled",
                    "The wake assertion ended before its timeout.",
                ));
            }
            Ok(json!({"awake_seconds":seconds,"finished":true}))
        }
        ["appearance", "get"] => {
            let mut appearance = applications::appearance(None)?;
            appearance["automatic"] = settings::MacBackend
                .read(settings::find("appearance.auto")?)?
                .value
                .unwrap_or(json!(false));
            Ok(appearance)
        }
        ["appearance", "set"] => {
            let mode = text(m, "mode");
            let operation = settings::set(
                "appearance.auto",
                if mode == "auto" { "true" } else { "false" },
                false,
            )?;
            if mode != "auto" {
                if let Err(e) = applications::appearance(Some(mode)) {
                    if let Some(id) = operation["operation"].as_str() {
                        if let Err(rollback) =
                            settings::Store::user()?.undo(&mut settings::MacBackend, id)
                        {
                            return Err(Error::new(
                                "rollback_failed",
                                format!("{e} Could not restore automatic appearance: {rollback}"),
                            ));
                        }
                    }
                    return Err(e);
                }
            }
            Ok(json!({"mode":mode,"operation":operation}))
        }
        ["appearance", "wallpaper"] => {
            native::call("wallpaper", json!({"path":absolute(text(m,"path"))?}))
        }
        ["settings", "list"] => Ok(json!(settings::CATALOG
            .iter()
            .filter(|s| exp || !s.experimental)
            .collect::<Vec<_>>())),
        ["settings", "get"] => settings::get(text(m, "key")),
        ["settings", "set"] => settings::set(text(m, "key"), text(m, "value"), exp),
        ["settings", "undo"] => {
            settings::Store::user()?.undo(&mut settings::MacBackend, text(m, "operation"))
        }
        ["profile", "save"] => {
            settings::save_profile(Path::new(text(m, "file")), &strings(m, "keys"), exp)
        }
        ["profile", action] => {
            let profile = settings::read_profile(Path::new(text(m, "file")))?;
            match *action {
                "show" => Ok(serde_json::to_value(profile)?),
                "diff" => settings::diff(&profile),
                "apply" => settings::Store::user()?.apply(
                    &mut settings::MacBackend,
                    &profile.settings,
                    exp,
                ),
                _ => Err(Error::invalid("Unknown profile operation.")),
            }
        }
        ["lock"] => native::call("session.lock", json!({})),
        ["sleep"] => {
            process::tool("/usr/bin/pmset", &["sleepnow"])?;
            Ok(json!({"requested":"Sleep"}))
        }
        ["screensaver"] => applications::open("com.apple.ScreenSaver.Engine"),
        ["restart"] | ["shutdown"] => {
            let action = p[0];
            confirm(
                m,
                &format!(
                    "{} this Mac now?",
                    if action == "restart" {
                        "Restart"
                    } else {
                        "Shut down"
                    }
                ),
            )?;
            process::elevated(
                "/sbin/shutdown",
                &[
                    if action == "restart" { "-r" } else { "-h" }.into(),
                    "now".into(),
                ],
            )?;
            Ok(json!({"requested":action}))
        }
        ["capture", "screenshot"] => screenshot(m),
        ["capture", "ocr"] => native::call("ocr", json!({"path":absolute(text(m,"path"))?})),
        ["clipboard", "write"] => {
            let content = if let Some(text) = m.get_one::<String>("text") {
                text.clone()
            } else {
                let mut s = String::new();
                io::stdin()
                    .take(16 * 1024 * 1024 + 1)
                    .read_to_string(&mut s)?;
                if s.len() > 16 * 1024 * 1024 {
                    return Err(Error::invalid("Clipboard input exceeds 16 MiB."));
                }
                s
            };
            native::call("clipboard.write", json!({"text":content}))
        }
        ["clipboard", action] => native::call(&format!("clipboard.{action}"), json!({})),
        ["system", "info"] => system_info(),
        ["system", "memory"] => result_text(process::tool("/usr/bin/vm_stat", &[])?),
        ["system", "cpu"] => result_text(process::tool("/usr/sbin/sysctl", &["-n", "vm.loadavg"])?),
        ["system", "processes"] => result_text(process::tool(
            "/bin/ps",
            &["-Aceo", "pid,pcpu,pmem,comm", "-r"],
        )?),
        ["disk", "list"] => {
            let t = process::tool("/usr/sbin/diskutil", &["list", "-plist"])?;
            native::call("plist.decode", json!({"text":t}))
        }
        ["disk", "usage"] => result_text(process::tool("/bin/df", &["-h"])?),
        ["disk", "eject"] => {
            let target = text(m, "target");
            if target.starts_with('-') || target.is_empty() {
                return Err(Error::invalid(
                    "Provide a disk identifier or mounted volume path.",
                ));
            }
            result_text(process::tool("/usr/sbin/diskutil", &["eject", target])?)
        }
        ["backup", "status"] => result_text(process::tool("/usr/bin/tmutil", &["status"])?),
        ["backup", "start"] => {
            process::tool("/usr/bin/tmutil", &["startbackup", "--auto"])?;
            Ok(json!({"requested":"Time Machine backup"}))
        }
        ["files", "size"] => size(&absolute(text(m, "path"))?),
        ["files", "largest"] => largest(&absolute(text(m, "path"))?, number(m, "limit") as usize),
        ["trash", "size"] => {
            let home = std::env::var_os("HOME")
                .ok_or_else(|| Error::new("configuration_error", "HOME is not set."))?;
            size(&PathBuf::from(home).join(".Trash"))
        }
        ["trash", "empty"] => {
            confirm(
                m,
                "Permanently empty your Trash, including trashed files on connected volumes?",
            )?;
            applications::empty_trash()
        }
        ["update", "list"] => result_text(process::run(
            "/usr/sbin/softwareupdate",
            &["--list".into()],
            None,
            Duration::from_secs(300),
        )?),
        ["update", "install"] => {
            if m.get_flag("json") {
                return Err(Error::invalid(
                    "Update installation uses interactive progress output. Omit --json.",
                ));
            }
            let label = text(m, "label");
            if label.starts_with('-') {
                return Err(Error::invalid(
                    "Provide one update label from mac update list.",
                ));
            }
            confirm(m, &format!("Install macOS update '{label}'?"))?;
            // Authentication and progress stay in the user's terminal; the command never requests a password itself.
            if !io::stdin().is_terminal() {
                return Err(Error::new(
                    "authentication_required",
                    "Run software update installation in an interactive terminal.",
                ));
            }
            let status = std::process::Command::new("/usr/bin/sudo")
                .args(["/usr/sbin/softwareupdate", "--install", label])
                .env("LC_ALL", "C")
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()?;
            if !status.success() {
                return Err(Error::new(
                    "command_failed",
                    "The macOS update did not complete.",
                ));
            }
            Ok(json!({"installed":label}))
        }
        ["weather"] | ["weather", "open"] => applications::open("weather"),
        ["apps", "list"] => applications::list(),
        ["apps", "open"] => applications::open(text(m, "name")),
        ["calendar", action] | ["reminders", action] => {
            if *action == "delete" {
                confirm(m, &format!("Delete {} item '{}'?", p[0], text(m, "id")))?;
            }
            let fields = match *action {
                "add" if p[0] == "calendar" => vec!["title", "start", "end", "calendar"],
                "add" => vec!["title", "list"],
                "events" => vec!["start", "end"],
                "delete" if p[0] == "calendar" => vec!["id", "start"],
                "delete" | "complete" => vec!["id"],
                "search" => vec!["query"],
                "list" if p[0] == "reminders" => vec!["list"],
                _ => vec![],
            };
            if *action == "events" || (*action == "add" && p[0] == "calendar") {
                validate_dates(text(m, "start"), text(m, "end"))?;
            }
            native::call(&format!("{}.{action}", p[0]), values(m, &fields))
        }
        ["notes", action] => {
            let fields = match *action {
                "search" => vec!["query"],
                "read" => vec!["id"],
                "add" => vec!["title", "body", "folder"],
                _ => vec![],
            };
            applications::notes(
                action,
                &fields
                    .iter()
                    .map(|f| text(m, f).to_owned())
                    .collect::<Vec<_>>(),
            )
        }
        [action @ ("play" | "pause")] => native::call(&format!("media.{action}"), json!({})),
        ["music", action] => applications::music(action),
        ["shortcuts", "list"] => result_text(process::tool("/usr/bin/shortcuts", &["list"])?),
        ["shortcuts", "run"] => {
            let mut args = vec!["run".into(), text(m, "name").into()];
            if text(m, "name").starts_with('-') {
                return Err(Error::invalid(
                    "A shortcut name must not start with a hyphen.",
                ));
            }
            for name in ["input", "output"] {
                if let Some(path) = m.get_one::<String>(name) {
                    args.push(format!("--{name}-path"));
                    args.push(path.clone());
                }
            }
            result_text(process::run(
                "/usr/bin/shortcuts",
                &args,
                None,
                Duration::from_secs(300),
            )?)
        }
        _ => Err(Error::invalid("Unknown command. Run mac --help.")),
    }
}
fn absolute(path: &str) -> Result<PathBuf> {
    let p = if path == "~" || path.starts_with("~/") {
        PathBuf::from(
            std::env::var_os("HOME")
                .ok_or_else(|| Error::new("configuration_error", "HOME is not set."))?,
        )
        .join(path.strip_prefix("~/").unwrap_or(""))
    } else {
        PathBuf::from(path)
    };
    Ok(if p.is_absolute() {
        p
    } else {
        std::env::current_dir()?.join(p)
    })
}
fn validate_dates(start: &str, end: &str) -> Result<()> {
    let start = chrono::DateTime::parse_from_rfc3339(start)
        .map_err(|_| Error::invalid("Start must be an RFC3339 timestamp including a time zone."))?;
    let end = chrono::DateTime::parse_from_rfc3339(end)
        .map_err(|_| Error::invalid("End must be an RFC3339 timestamp including a time zone."))?;
    if end <= start {
        return Err(Error::invalid("End must be after start."));
    }
    if end - start > chrono::Duration::days(366 * 4) {
        return Err(Error::invalid(
            "A calendar date range cannot exceed four years.",
        ));
    }
    Ok(())
}
fn dns(action: &str, m: &ArgMatches) -> Result<Value> {
    if action == "flush" {
        process::elevated("/usr/bin/dscacheutil", &["-flushcache".into()])?;
        process::elevated("/usr/bin/killall", &["-HUP".into(), "mDNSResponder".into()])?;
        return Ok(json!({"flushed":true}));
    }
    let service = text(m, "service");
    if service.starts_with('-') {
        return Err(Error::invalid("Invalid network service name."));
    }
    if action == "get" {
        return result_text(process::tool(
            "/usr/sbin/networksetup",
            &["-getdnsservers", service],
        )?);
    }
    let servers = if action == "reset" {
        vec!["Empty".into()]
    } else {
        strings(m, "servers")
    };
    if action != "reset" {
        for server in &servers {
            server
                .parse::<std::net::IpAddr>()
                .map_err(|_| Error::invalid(format!("Invalid DNS IP address: {server}")))?;
        }
    }
    let mut args = vec!["-setdnsservers".into(), service.into()];
    args.extend(servers);
    process::elevated("/usr/sbin/networksetup", &args)?;
    result_text(process::tool(
        "/usr/sbin/networksetup",
        &["-getdnsservers", service],
    )?)
}
fn screenshot(m: &ArgMatches) -> Result<Value> {
    let clipboard = m.get_flag("clipboard");
    let path = if let Some(p) = m.get_one::<String>("file") {
        absolute(p)?
    } else {
        std::env::current_dir()?.join(format!(
            "Screenshot-{}.png",
            chrono::Local::now().format("%Y%m%d-%H%M%S%.3f")
        ))
    };
    if !clipboard && fs::symlink_metadata(&path).is_ok() {
        return Err(Error::new(
            "already_exists",
            "The screenshot file already exists. Choose a new path.",
        ));
    }
    let temporary = if clipboard {
        tempfile::Builder::new()
            .prefix("mac-cli-capture-")
            .tempdir()?
    } else {
        tempfile::Builder::new()
            .prefix(".mac-cli-capture-")
            .tempdir_in(
                path.parent()
                    .ok_or_else(|| Error::invalid("A screenshot directory is required."))?,
            )?
    };
    let captured = temporary.path().join("capture.png");
    let mut args = vec!["-x".into(), "-t".into(), "png".into()];
    match text(m, "mode") {
        "region" => args.push("-i".into()),
        "window" => {
            args.push("-i".into());
            args.push("-w".into());
        }
        _ => {}
    }
    args.push(captured.to_string_lossy().into_owned());
    process::run(
        "/usr/sbin/screencapture",
        &args,
        None,
        Duration::from_secs(120),
    )?;
    if !captured.is_file() || fs::metadata(&captured)?.len() == 0 {
        return Err(Error::new("cancelled","No screenshot was saved. Capture may have been cancelled or permission may be missing."));
    }
    if clipboard {
        native::call("clipboard.image", json!({"path":captured}))?;
    } else {
        publish_capture(&captured, &path)?;
    }
    Ok(if clipboard {
        json!({"copied":true})
    } else {
        json!({"saved":path})
    })
}
fn publish_capture(source: &Path, destination: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(source, fs::Permissions::from_mode(0o600))?;
    }
    fs::hard_link(source, destination).map_err(|e| {
        if e.kind() == io::ErrorKind::AlreadyExists {
            Error::new(
                "already_exists",
                "The screenshot destination already exists; nothing was overwritten.",
            )
        } else {
            e.into()
        }
    })?;
    Ok(())
}
fn system_info() -> Result<Value> {
    let mut value = json!({"os":process::tool("/usr/bin/sw_vers",&["-productVersion"])?.trim(),"architecture":"arm64"});
    for (key, name) in [
        ("model", "hw.model"),
        ("memory_bytes", "hw.memsize"),
        ("logical_cpus", "hw.logicalcpu"),
        ("load_average", "vm.loadavg"),
    ] {
        value[key] = match process::tool("/usr/sbin/sysctl", &["-n", name]) {
            Ok(v) => json!(v.trim()),
            Err(_) => Value::Null,
        };
    }
    Ok(value)
}
fn size(path: &Path) -> Result<Value> {
    result_text(process::tool(
        "/usr/bin/du",
        &["-sk", &path.to_string_lossy()],
    )?)
}
fn largest(root: &Path, limit: usize) -> Result<Value> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::<(u64, PathBuf)>::new();
    let mut skipped = 0;
    if !root.is_dir() {
        return Err(Error::invalid("Provide an existing directory."));
    }
    while let Some(path) = pending.pop() {
        let entries = match fs::read_dir(&path) {
            Ok(e) => e,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };
            let meta = match fs::symlink_metadata(entry.path()) {
                Ok(m) => m,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };
            if meta.is_symlink() {
                continue;
            }
            if meta.is_dir() {
                pending.push(entry.path());
            } else if meta.is_file() {
                files.push((meta.len(), entry.path()));
                files.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
                files.truncate(limit);
            }
        }
    }
    Ok(
        json!({"files":files.into_iter().map(|(bytes,path)|json!({"path":path,"bytes":bytes})).collect::<Vec<_>>(),"unreadable_entries":skipped}),
    )
}
fn menu() -> Result<Value> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(Error::new(
            "terminal_required",
            "The menu requires an interactive terminal.",
        ));
    }
    let mut command = cli::command();
    command.build();
    let leaves = cli::leaves(&command, vec![])
        .into_iter()
        .filter(|(p, _)| p[0] != "menu")
        .collect::<Vec<_>>();
    loop {
        let labels = leaves
            .iter()
            .map(|(p, c)| {
                format!(
                    "{} — {}",
                    p.join(" "),
                    c.get_about().map(ToString::to_string).unwrap_or_default()
                )
            })
            .collect::<Vec<_>>();
        let selected = dialoguer::FuzzySelect::new()
            .with_prompt("mac-cli · Search commands · Esc to exit")
            .items(&labels)
            .interact_opt()
            .map_err(|e| Error::new("terminal_error", e.to_string()))?;
        let Some(index) = selected else {
            return Ok(json!({"message":"Menu closed."}));
        };
        let (path, leaf) = &leaves[index];
        let mut argv = vec!["mac".to_string()];
        argv.extend(path.iter().cloned());
        let mut cancelled = false;
        for arg in leaf.get_positionals() {
            let optional = !arg.is_required_set();
            let input: String = dialoguer::Input::new()
                .with_prompt(format!(
                    "{}{}",
                    arg.get_id(),
                    if optional { " (optional)" } else { "" }
                ))
                .allow_empty(optional)
                .interact_text()
                .map_err(|e| Error::new("terminal_error", e.to_string()))?;
            if input.is_empty() {
                continue;
            }
            if arg.get_num_args().is_some_and(|n| n.max_values() > 1) {
                argv.push(input);
                loop {
                    let next: String = dialoguer::Input::new()
                        .with_prompt(format!("Another {} (blank to finish)", arg.get_id()))
                        .allow_empty(true)
                        .interact_text()
                        .map_err(|e| Error::new("terminal_error", e.to_string()))?;
                    if next.is_empty() {
                        break;
                    }
                    argv.push(next);
                }
            } else {
                argv.push(input);
            }
        }
        for arg in leaf.get_arguments().filter(|a| a.get_long().is_some()) {
            let name = arg.get_id().as_str();
            if ["help", "version", "json", "plain", "yes"].contains(&name) {
                continue;
            }
            if matches!(arg.get_action(), clap::ArgAction::SetTrue) {
                if dialoguer::Confirm::new()
                    .with_prompt(format!("Enable --{name}?"))
                    .default(false)
                    .interact()
                    .map_err(|e| Error::new("terminal_error", e.to_string()))?
                {
                    argv.push(format!("--{name}"));
                }
            } else {
                let input: String = dialoguer::Input::new()
                    .with_prompt(format!("--{name} (optional)"))
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| Error::new("terminal_error", e.to_string()))?;
                if !input.is_empty() {
                    argv.push(format!("--{name}"));
                    argv.push(input);
                }
            }
        }
        match cli::command().try_get_matches_from(&argv) {
            Ok(matches) => {
                let mut m = &matches;
                while let Some((_, child)) = m.subcommand() {
                    m = child;
                }
                let result = execute(path, m);
                output::emit(&path.join(" "), result, false, false);
            }
            Err(error) => {
                eprintln!("{error}");
                cancelled = true;
            }
        }
        if !cancelled {
            let _ = io::stdout().flush();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calendar_dates_are_unambiguous() {
        assert!(validate_dates("2026-09-09T10:00:00+03:00", "2026-09-09T11:00:00+03:00").is_ok());
        assert!(validate_dates("tomorrow", "Friday").is_err());
        assert!(validate_dates("2026-09-09T12:00:00Z", "2026-09-09T11:00:00Z").is_err());
    }
    #[test]
    fn largest_skips_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("small"), "a").unwrap();
        fs::write(dir.path().join("large"), "abc").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(dir.path(), dir.path().join("loop")).unwrap();
        let result = largest(dir.path(), 1).unwrap();
        assert_eq!(result["files"][0]["bytes"], 3);
    }
    #[test]
    fn capture_commit_never_overwrites_a_racing_file_or_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("capture.png");
        fs::write(&source, b"image").unwrap();
        let destination = dir.path().join("existing.png");
        fs::write(&destination, b"keep").unwrap();
        assert_eq!(
            publish_capture(&source, &destination).unwrap_err().code,
            "already_exists"
        );
        assert_eq!(fs::read(&destination).unwrap(), b"keep");
        let target = dir.path().join("missing");
        let symlink = dir.path().join("symlink.png");
        std::os::unix::fs::symlink(&target, &symlink).unwrap();
        assert!(publish_capture(&source, &symlink).is_err());
        assert!(!target.exists());
        let new = dir.path().join("new.png");
        publish_capture(&source, &new).unwrap();
        assert_eq!(fs::read(new).unwrap(), b"image");
    }
}
