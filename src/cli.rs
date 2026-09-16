use clap::{Arg, ArgAction, Command};

fn flag(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name)
        .long(name)
        .help(help)
        .action(ArgAction::SetTrue)
}
fn opt(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name).long(name).help(help)
}
fn pos(name: &'static str) -> Arg {
    Arg::new(name).required(true)
}
fn leaf(name: &'static str, about: &'static str) -> Command {
    Command::new(name).about(about)
}
fn group(name: &'static str, about: &'static str) -> Command {
    leaf(name, about)
        .subcommand_required(true)
        .arg_required_else_help(true)
}
fn level(name: &'static str, about: &'static str, mute: bool) -> Command {
    let mut c = leaf(name, about)
        .args_conflicts_with_subcommands(true)
        .arg(
            Arg::new("percent")
                .help("Set an absolute percentage from 0 to 100")
                .value_parser(clap::value_parser!(u8).range(0..=100)),
        )
        .subcommand(leaf("get", "Read the current percentage"))
        .subcommand(
            leaf("set", "Set a percentage from 0 to 100")
                .arg(pos("percent").value_parser(clap::value_parser!(u8).range(0..=100))),
        )
        .subcommand(
            leaf("up", "Increase by percentage points").arg(
                Arg::new("step")
                    .default_value("5")
                    .value_parser(clap::value_parser!(u8).range(0..=100)),
            ),
        )
        .subcommand(
            leaf("down", "Decrease by percentage points").arg(
                Arg::new("step")
                    .default_value("5")
                    .value_parser(clap::value_parser!(u8).range(0..=100)),
            ),
        );
    if mute {
        for name in ["mute", "unmute", "toggle"] {
            c = c.subcommand(leaf(name, "Change the mute state"));
        }
    }
    match name {
        "volume" => c.visible_alias("vol"),
        "brightness" => c.visible_alias("br"),
        _ => c,
    }
}
fn switch(name: &'static str, about: &'static str) -> Command {
    group(name, about)
        .subcommand(leaf("get", "Read the current state"))
        .subcommand(leaf("on", "Enable"))
        .subcommand(leaf("off", "Disable"))
}

pub fn command() -> Command {
    Command::new("mac")
        .version(env!("CARGO_PKG_VERSION"))
        .about("An English-only command center for macOS 14+ on Apple Silicon")
        .arg(
            flag("json", "Emit a versioned JSON result")
                .global(true)
                .conflicts_with("plain"),
        )
        .arg(flag("plain", "Use plain output without decorative bars").global(true))
        .arg(flag("yes", "Confirm the explicitly requested destructive action").global(true))
        .arg(flag("experimental", "Allow experimental settings").global(true))
        .subcommand(leaf("status", "Show a system overview"))
        .subcommand(leaf("menu", "Browse and run commands interactively"))
        .subcommand(leaf("commands", "List commands and capability information"))
        .subcommand(leaf(
            "doctor",
            "Check platform, permissions, and hardware without requesting access",
        ))
        .subcommand(leaf(
            "battery",
            "Show battery charge, health, and power source",
        ))
        .subcommand(
            level("brightness", "Control display brightness", false).arg(
                opt("display", "Display ID; defaults to the built-in display")
                    .global(true)
                    .value_parser(clap::value_parser!(u32)),
            ),
        )
        .subcommand(
            group("display", "List displays and configure a display mode")
                .subcommand(leaf("list", "List connected displays"))
                .subcommand(
                    leaf("modes", "List modes for a display")
                        .arg(pos("id").value_parser(clap::value_parser!(u32))),
                )
                .subcommand(
                    leaf("set", "Set a listed mode on a display")
                        .arg(pos("id").value_parser(clap::value_parser!(u32)))
                        .arg(pos("mode").value_parser(clap::value_parser!(u32))),
                ),
        )
        .subcommand(level("volume", "Control output volume", true))
        .subcommand(
            group("audio", "Manage audio devices")
                .subcommand(
                    group("output", "Select an output device")
                        .subcommand(leaf("list", "List output devices"))
                        .subcommand(
                            leaf("set", "Select an output device by ID")
                                .arg(pos("id").value_parser(clap::value_parser!(u32))),
                        ),
                )
                .subcommand(
                    group("input", "Select an input device")
                        .subcommand(leaf("list", "List input devices"))
                        .subcommand(
                            leaf("set", "Select an input device by ID")
                                .arg(pos("id").value_parser(clap::value_parser!(u32))),
                        ),
                )
                .subcommand(level("gain", "Control input gain where supported", false)),
        )
        .subcommand(
            group("keyboard", "Control keyboard backlight and input sources")
                .subcommand(level("brightness", "Control keyboard backlight", false))
                .subcommand(
                    group("source", "Manage keyboard input sources")
                        .subcommand(leaf("list", "List selectable input sources"))
                        .subcommand(leaf("set", "Select an input source by ID").arg(pos("id"))),
                ),
        )
        .subcommand(
            group("wifi", "Control Wi-Fi")
                .subcommand(leaf("status", "Show Wi-Fi status"))
                .subcommand(leaf("on", "Enable Wi-Fi"))
                .subcommand(leaf("off", "Disable Wi-Fi"))
                .subcommand(leaf(
                    "scan",
                    "Scan nearby networks; may require Location Services",
                ))
                .subcommand(
                    leaf(
                        "connect",
                        "Connect to a Wi-Fi network; securely prompt for its password",
                    )
                    .arg(pos("ssid"))
                    .arg(flag("open", "Connect to an open network")),
                ),
        )
        .subcommand(
            group("bluetooth", "Control Bluetooth and paired devices")
                .subcommand(leaf("status", "Read Bluetooth power"))
                .subcommand(leaf("on", "Enable Bluetooth"))
                .subcommand(leaf("off", "Disable Bluetooth"))
                .subcommand(leaf("list", "List paired devices"))
                .subcommand(leaf("scan", "Discover nearby Bluetooth devices"))
                .subcommand(
                    leaf("connect", "Connect a paired device by address").arg(pos("address")),
                )
                .subcommand(
                    leaf("disconnect", "Disconnect a device by address").arg(pos("address")),
                ),
        )
        .subcommand(
            group("network", "Inspect network configuration")
                .subcommand(leaf("ip", "List local interface addresses"))
                .subcommand(leaf("interfaces", "List network interfaces"))
                .subcommand(leaf("ports", "List listening TCP ports"))
                .subcommand(
                    group("dns", "Manage DNS servers")
                        .subcommand(
                            leaf("get", "Read DNS servers for a network service")
                                .arg(pos("service")),
                        )
                        .subcommand(
                            leaf("set", "Set DNS servers for a network service")
                                .arg(pos("service"))
                                .arg(pos("servers").num_args(1..)),
                        )
                        .subcommand(
                            leaf("reset", "Return a network service to automatic DNS")
                                .arg(pos("service")),
                        )
                        .subcommand(leaf("flush", "Flush the system DNS cache")),
                ),
        )
        .subcommand(
            group("power", "Control power policy")
                .subcommand(leaf("status", "Read power settings"))
                .subcommand(switch("low-power", "Control low power mode"))
                .subcommand(
                    leaf(
                        "sleep-after",
                        "Set sleep timers in minutes; zero disables the timer",
                    )
                    .arg(pos("minutes").value_parser(clap::value_parser!(u32).range(0..=1440)))
                    .arg(
                        opt("target", "Timer to change")
                            .value_parser(["system", "display"])
                            .default_value("system"),
                    ),
                )
                .subcommand(
                    leaf("awake", "Keep the system awake until the timeout or Ctrl-C")
                        .arg(pos("seconds").value_parser(clap::value_parser!(u32).range(1..))),
                ),
        )
        .subcommand(
            group("appearance", "Control appearance and wallpaper")
                .subcommand(leaf("get", "Read the current appearance"))
                .subcommand(
                    leaf("set", "Set the appearance")
                        .arg(pos("mode").value_parser(["light", "dark", "auto"])),
                )
                .subcommand(leaf("wallpaper", "Set wallpaper on all screens").arg(pos("path"))),
        )
        .subcommand(settings_command())
        .subcommand(
            group("profile", "Save, compare, and apply selected settings")
                .subcommand(
                    leaf("save", "Save selected settings to a TOML file")
                        .arg(pos("file"))
                        .arg(pos("keys").num_args(1..)),
                )
                .subcommand(leaf("show", "Read a profile").arg(pos("file")))
                .subcommand(
                    leaf("diff", "Compare a profile with current settings").arg(pos("file")),
                )
                .subcommand(
                    leaf("apply", "Apply a profile with rollback on failure").arg(pos("file")),
                ),
        )
        .subcommands(
            ["lock", "sleep", "screensaver", "restart", "shutdown"]
                .map(|n| leaf(n, "Control the current macOS session")),
        )
        .subcommand(
            group("capture", "Capture the screen or recognize text")
                .subcommand(
                    leaf("screenshot", "Capture a screen, region, or window")
                        .arg(
                            opt("mode", "Capture mode")
                                .value_parser(["screen", "region", "window"])
                                .default_value("screen"),
                        )
                        .arg(opt("file", "Output PNG path").conflicts_with("clipboard"))
                        .arg(flag("clipboard", "Copy the image to the clipboard")),
                )
                .subcommand(leaf("ocr", "Recognize text in an image file").arg(pos("path"))),
        )
        .subcommand(
            group("clipboard", "Read, write, or clear the text clipboard")
                .subcommand(leaf("read", "Read clipboard text"))
                .subcommand(
                    leaf("write", "Write text; reads stdin if text is omitted")
                        .arg(Arg::new("text")),
                )
                .subcommand(leaf("clear", "Clear the clipboard")),
        )
        .subcommand(
            group("system", "Inspect system resources")
                .subcommand(leaf("info", "Show hardware and OS information"))
                .subcommand(leaf("memory", "Show virtual memory statistics"))
                .subcommand(leaf("cpu", "Show CPU load averages"))
                .subcommand(leaf("processes", "List processes by CPU use")),
        )
        .subcommand(
            group("disk", "Inspect and eject volumes")
                .subcommand(leaf("list", "List disks and partitions"))
                .subcommand(leaf("usage", "Show mounted filesystem usage"))
                .subcommand(leaf("eject", "Safely eject a volume or disk").arg(pos("target"))),
        )
        .subcommand(
            group("backup", "Inspect and start Time Machine backups")
                .subcommand(leaf("status", "Read Time Machine backup status"))
                .subcommand(leaf("start", "Start a Time Machine backup")),
        )
        .subcommand(
            group("files", "Inspect file sizes without deleting files")
                .subcommand(leaf("size", "Calculate the size of a file or folder").arg(pos("path")))
                .subcommand(
                    leaf("largest", "List largest files under a folder")
                        .arg(pos("path"))
                        .arg(
                            opt("limit", "Maximum number of results")
                                .default_value("20")
                                .value_parser(clap::value_parser!(u32).range(1..=1000)),
                        ),
                ),
        )
        .subcommand(
            group("trash", "Inspect or empty your Trash")
                .subcommand(leaf("size", "Read Trash size"))
                .subcommand(leaf("empty", "Empty your Trash after confirmation")),
        )
        .subcommand(
            group("update", "Manage macOS software updates")
                .subcommand(leaf("list", "List available macOS updates"))
                .subcommand(
                    leaf("install", "Install one explicitly named update").arg(pos("label")),
                ),
        )
        .subcommand(
            leaf("weather", "Open the built-in Weather app")
                .subcommand(leaf("open", "Open Weather")),
        )
        .subcommand(
            group("apps", "Discover and open applications and utilities")
                .subcommand(leaf(
                    "list",
                    "List installed applications and utility aliases",
                ))
                .subcommand(
                    leaf("open", "Open an application by name, alias, or bundle ID")
                        .arg(pos("name")),
                ),
        )
        .subcommand(
            group("calendar", "Read and manage Calendar events")
                .subcommand(leaf("list", "List calendars"))
                .subcommand(leaf("today", "Read today's events in the local time zone"))
                .subcommand(
                    leaf("events", "Read an RFC3339 date range")
                        .arg(pos("start"))
                        .arg(pos("end")),
                )
                .subcommand(
                    leaf("add", "Create an event with RFC3339 start and end")
                        .arg(pos("title"))
                        .arg(pos("start"))
                        .arg(pos("end"))
                        .arg(opt(
                            "calendar",
                            "Calendar ID; defaults to the default calendar",
                        )),
                )
                .subcommand(
                    leaf("delete", "Delete one event occurrence by ID")
                        .arg(pos("id"))
                        .arg(opt(
                            "start",
                            "RFC3339 occurrence start; required for recurring events",
                        )),
                ),
        )
        .subcommand(
            group("reminders", "Read and manage Reminders")
                .subcommand(leaf("lists", "List reminder lists"))
                .subcommand(leaf("list", "List reminders").arg(opt("list", "Reminder list ID")))
                .subcommand(leaf("search", "Search reminder titles").arg(pos("query")))
                .subcommand(
                    leaf("add", "Create a reminder")
                        .arg(pos("title"))
                        .arg(opt("list", "Reminder list ID")),
                )
                .subcommand(leaf("complete", "Complete a reminder by ID").arg(pos("id")))
                .subcommand(leaf("delete", "Delete a reminder by ID").arg(pos("id"))),
        )
        .subcommand(
            group("notes", "Read and create Notes")
                .subcommand(leaf("folders", "List note folders"))
                .subcommand(leaf("list", "List note titles and IDs"))
                .subcommand(leaf("search", "Search note titles").arg(pos("query")))
                .subcommand(leaf("read", "Read a note by ID").arg(pos("id")))
                .subcommand(
                    leaf("add", "Create a plain-text note")
                        .arg(pos("title"))
                        .arg(pos("body"))
                        .arg(opt("folder", "Target folder ID")),
                ),
        )
        .subcommand(leaf("play", "Resume the system's current media session"))
        .subcommand(leaf("pause", "Pause the system's current media session"))
        .subcommand(
            group("music", "Control the Music app").subcommands(
                ["status", "play", "pause", "toggle", "next", "previous"]
                    .map(|n| leaf(n, "Read or control Music playback")),
            ),
        )
        .subcommand(
            group("shortcuts", "List and run Apple Shortcuts")
                .subcommand(leaf("list", "List available shortcuts"))
                .subcommand(
                    leaf("run", "Run a named shortcut")
                        .arg(pos("name"))
                        .arg(opt("input", "Input file"))
                        .arg(opt("output", "Output file")),
                ),
        )
}
fn settings_command() -> Command {
    group(
        "settings",
        "Browse and change the typed macOS settings catalog",
    )
    .subcommand(leaf(
        "list",
        "List settings; include experimental entries with --experimental",
    ))
    .subcommand(leaf("get", "Read one setting").arg(pos("key")))
    .subcommand(
        leaf("set", "Change one setting and record its previous value")
            .arg(pos("key"))
            .arg(pos("value")),
    )
    .subcommand(
        leaf(
            "undo",
            "Undo a recorded operation; refuses conflicting newer changes",
        )
        .arg(pos("operation")),
    )
}

pub fn leaves(c: &Command, prefix: Vec<String>) -> Vec<(Vec<String>, Command)> {
    let mut result = Vec::new();
    if !prefix.is_empty() && !c.is_subcommand_required_set() {
        result.push((prefix.clone(), c.clone()));
    }
    for child in c.get_subcommands().filter(|s| s.get_name() != "help") {
        let mut path = prefix.clone();
        path.push(child.get_name().to_owned());
        result.extend(leaves(child, path));
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cli_is_consistent() {
        command().debug_assert();
    }
    #[test]
    fn percentage_validation() {
        assert!(command()
            .try_get_matches_from(["mac", "volume", "set", "101"])
            .is_err());
        assert!(command()
            .try_get_matches_from(["mac", "brightness", "set", "-1"])
            .is_err());
        assert!(command()
            .try_get_matches_from(["mac", "keyboard", "brightness", "set", "0"])
            .is_ok());
        assert!(command()
            .try_get_matches_from(["mac", "--json", "reminders", "add", "Çalışma"])
            .is_ok());
    }
    #[test]
    fn level_shorthand() {
        for name in ["vol", "volume", "br", "brightness"] {
            for value in ["0", "50", "100"] {
                let m = command()
                    .try_get_matches_from(["mac", name, value])
                    .unwrap();
                assert_eq!(
                    m.subcommand().unwrap().1.get_one::<u8>("percent").copied(),
                    Some(value.parse().unwrap())
                );
            }
            for value in ["101", "-1", "50.5", "loud"] {
                assert!(command()
                    .try_get_matches_from(["mac", name, value])
                    .is_err());
            }
            assert!(command()
                .try_get_matches_from(["mac", name, "50", "up"])
                .is_err());
            assert!(command().try_get_matches_from(["mac", name, "up"]).is_ok());
        }
        let m = command()
            .try_get_matches_from(["mac", "vol", "50", "--json"])
            .unwrap();
        assert_eq!(m.subcommand_name(), Some("volume"));
    }
    #[test]
    fn catalog_covers_features() {
        let paths: Vec<_> = leaves(&command(), vec![])
            .into_iter()
            .map(|(p, _)| p.join(" "))
            .collect();
        for name in [
            "volume set",
            "play",
            "pause",
            "weather",
            "calendar today",
            "settings undo",
            "keyboard brightness set",
            "notes read",
            "menu",
        ] {
            assert!(paths.contains(&name.to_string()), "{name}");
        }
    }
}
