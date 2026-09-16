# Compatibility and verification

Target: macOS 14 or later, Apple Silicon only. This is a compatibility target,
not a statement that every feature has been manually verified on every release.

## Local verification — 2026-09-10

Host: Apple M1 Max, macOS 26.6.2. The lid was closed, with a generic external
display active. No private application data was read as part of automated tests.

| Check | Result |
| --- | --- |
| Rust debug build | Passed |
| Rust release build | Passed |
| Unit tests | 28 passed |
| Release script tests | 3 passed |
| Dependency security audit | No known vulnerabilities or warnings |
| CLI integration tests | 6 passed |
| Clippy with warnings denied | Passed |
| Rust formatting | Passed |
| Every registered command's help | 127 command paths passed |
| English help under Turkish locale environment | Passed |
| JSON discovery and settings catalog | Passed |
| Menu launch and Escape exit in a terminal | Passed |
| Battery reading | Passed |
| System information, local interfaces, power configuration | Passed |
| Application discovery and preference reading | Passed |
| Weather launch | Passed through LaunchServices |
| Disk listing | Passed outside the execution sandbox |
| Keyboard source enumeration | Passed |
| Audio device discovery and volume reading | Passed outside the execution sandbox |
| Volume change and exact previous-level restoration | Passed on the current output device |
| Display brightness | Not verified: only an unsupported external display was active |
| Keyboard brightness reading | Passed outside the execution sandbox |
| Keyboard brightness writing | Not verified: lid closed; now rejected before writing |
| Calendar / Reminders consent and data operations | Not exercised against personal data |
| Notes / Music Automation operations | Implemented; personal-data/UI behavior not yet verified |
| Notes / Music scripting syntax | Compiled successfully without accessing app data |
| Wi-Fi/Bluetooth state changes | Not exercised; do not disrupt the active connection |
| Administrator operations, deletion, reboot, update install | Not exercised on this machine |
| macOS 14 and 15 runtime | Not locally verified |
| Linux rejection binary | CI test provided; no local Linux runtime was available |
| Direct mac command in a fresh login shell | Passed: version and battery JSON |
| Installer overwrite protection | Passed in a temporary directory |
| Homebrew formula generator | Syntax/checksum checked using a test URL |
| Local arm64 archive | Packaged, signature verified, extracted binary launched; minimum OS 14.0 confirmed |

An initial keyboard write attempt did not change the backlight. The original
value was restored and confirmed. The system reported AppleClamshellState=true;
the backend now checks that condition and requests opening the lid. A closed
lid must not be used as evidence that the open-lid backend works or fails.

Sandboxed processes may be unable to enumerate displays, audio devices, or
DiskManagement services even when the normal desktop session can. Hardware
checks must run from the actual logged-in terminal session.

## Release acceptance

Before claiming full hardware support, open the lid and run:

    python3 scripts/hardware-smoke.py --binary target/release/mac --allow-changes

Also verify the visible brightness change, the value after process exit,
automatic brightness behavior, and restoration. A programmatic read-back alone
does not prove the physical panel or keyboard visibly changed.

Test permissions in both granted and denied states using disposable Calendar,
Reminders, and Notes items. Confirm deleting an event affects only its selected
occurrence. Test screenshots and clipboard with disposable content.

CI is configured for macOS 14, 15, 26 and Linux. Hosted CI cannot validate laptop
hardware, desktop permission prompts, or personal application integrations.
The workflows have not been run remotely in this workspace.

Intel and general DDC/CI displays are out of scope. Hardware-specific APIs can
change between macOS updates. Unsupported operations return an error; they do
not silently install another utility or report a fabricated value.

## System media controls — 2026-09-16

`mac play` and `mac pause` were verified against a temporary, silent MediaPlayer
Now Playing session on macOS 27.0 (Apple Silicon). The test player received two pause
commands and remained paused, then received two play commands and remained
playing. The test player was terminated afterwards. This confirms system routing
and distinct, repeatable commands; it does not certify every browser or player.
All 29 unit and 6 integration tests passed, including help for 129 command paths.
Clippy passed with warnings denied. Other macOS versions remain unverified.

The backend dynamically resolves MRMediaRemoteSendCommand. Apps must participate
in macOS Now Playing. Only the system-selected session is targeted. A successful
response means macOS accepted the request, not that a player state was read back.
Private API availability is reported by `mac doctor` as media_control_api.
