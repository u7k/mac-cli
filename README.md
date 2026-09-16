# mac-cli

Control your Mac from the terminal.

Built with Rust for macOS 14+ on Apple Silicon. All commands and output are in English.

## Install

With Rust and Xcode Command Line Tools installed, run from the project directory:

```sh
cargo build --release --locked
sh scripts/install.sh
```

The installer places `mac` in `~/.local/bin` and refuses to overwrite an existing
command. If your shell cannot find it, add that directory to your `PATH`.

## Try it

```sh
mac                  # System overview
mac menu             # Searchable command menu
mac battery          # Battery charge and health
mac vol 50           # Set volume to 50%
mac br 65            # Set display brightness to 65%
mac keyboard br 30   # Set keyboard brightness to 30%
mac pause            # Pause the current system media session
mac play             # Resume playback
mac weather          # Open Weather
mac calendar today   # Today's events
mac doctor           # Check hardware support and permissions
```

`vol` and `br` are short forms of `volume` and `brightness`. Percentages range
from 0 to 100. Use `mac vol up` or `mac vol down` to change volume by five points.
Brightness controls require compatible hardware; open the MacBook lid for its
keyboard backlight. Playback commands target the session selected by macOS.

## Explore

```sh
mac --help
mac commands
mac settings list
mac settings set dock.autohide true
mac battery --json
```

The CLI also covers networking, power, audio devices, screenshots, clipboard,
disks, profiles, and built-in apps. Use `--help` on any command, `--json` for
structured output, or `--plain` to disable decorative bars.

Some features need macOS permissions. Destructive actions ask for confirmation.
Run `mac` as your normal user; administrator operations request authentication
when needed.

## Documentation

- [Command reference](docs/commands.md)
- [Settings, apps, and permissions](docs/usage.md)
- [Compatibility and verification](docs/compatibility.md)
- [Development and releases](docs/releasing.md)
- [Architecture](docs/architecture.md) and [security](SECURITY.md)

Inspired by [Omarchy CLI](https://omarchy.org/manual/omarchy-cli/) and
[Mac-CLI](https://github.com/guarinogabriel/mac-cli). MIT licensed.
