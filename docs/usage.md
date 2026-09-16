# Usage details

## Settings and profiles

    mac settings list
    mac settings list --experimental
    mac settings get dock.autohide
    mac settings set dock.autohide true
    mac settings set dock.size 48
    mac settings set dock.autohide-delay 0.1 --experimental
    mac profile save desktop.toml dock.autohide dock.size finder.path-bar
    mac profile show desktop.toml
    mac profile diff desktop.toml
    mac profile apply desktop.toml
    mac settings undo OPERATION_ID

The catalog describes types, ranges, restart requirements, and experimental
settings. A missing override means that macOS chooses the default. Some changes, including
keyboard and trackpad preferences, require an application restart or sign-in.

Profiles are TOML files containing only explicitly selected catalog settings.
Saving refuses to replace an existing file. Experimental entries require
--experimental when applied.

Settings changes are journaled under $XDG_STATE_HOME/mac-cli or
~/.local/state/mac-cli with private permissions. All entries are validated before
writing. Failures trigger rollback, including restoring the absence of a key.
Undo refuses unrelated newer values. Dock/Finder restarts are combined per
operation. Pending journals from an interrupted process can be recovered with
mac settings undo OPERATION_ID.

## Built-in apps

    mac weather
    mac apps list
    mac apps open activity-monitor
    mac apps open disk-utility
    mac apps open com.apple.Console
    mac calendar list
    mac calendar today
    mac calendar add "Review" 2026-09-10T10:00:00+03:00 2026-09-10T11:00:00+03:00
    mac reminders lists
    mac reminders add "Check backups"
    mac reminders complete REMINDER_ID
    mac notes folders
    mac notes search "Project"
    mac notes add "Project" "First steps"
    mac pause
    mac play
    mac music status
    mac music toggle
    mac shortcuts list
    mac shortcuts run "My Focus Shortcut"

`mac pause` and `mac play` target the media session selected by macOS Now Playing,
including compatible browsers and third-party players. Only one session is
controlled. Repeating pause keeps playback paused. With no session, the request
may have no effect. Output reports whether the command was sent.
`mac music ...` targets Apple Music directly.

`mac weather` opens Apple's Weather app. To control Focus or run other custom
actions, create a Shortcut and run it with `mac shortcuts run`.

Calendar times require RFC3339 with an explicit offset. "Today" uses the local
time zone. Delete removes one identified event occurrence or reminder, after
confirmation. Recurring event deletion requires --start with the occurrence
start returned by calendar events; a series is never deleted implicitly.
Notes access covers plain text; locked notes and attachment export
are not supported.

## Permissions and output

Run mac doctor to inspect support without requesting application-data access.
Feature commands request permission only when needed:

| Feature | macOS permission |
| --- | --- |
| Calendar / Reminders | Calendars / Reminders |
| Notes / Music / appearance / empty Trash | Automation for the selected app |
| Wi-Fi names and scanning | Location Services for the responsible terminal |
| Bluetooth devices | Bluetooth |
| Screenshots | Screen Recording |

macOS may attribute command-line permissions to your terminal or signing
identity. Rebuilding an ad-hoc binary can change that identity and require
permission again.

--yes confirms only the destructive operation explicitly requested. Without it,
non-interactive deletion, shutdown, or update installation fails before accessing
the affected data. It does not bypass OS permissions or administrator authentication.

JSON results have schema_version, command, ok, and either data or error.
Feature failures exit 1; argument errors exit 2; platform failures exit 78.
Diagnostic system utilities may return their English output under data.text.
