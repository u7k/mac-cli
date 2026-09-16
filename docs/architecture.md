# Architecture

## Startup

main calls platform::check first. The check uses the compiled OS/architecture
and, on arm64 macOS, sysctl kern.osproductversion. It has no configuration writes,
child processes, or feature bridge calls. Non-macOS builds include only the
unsupported-platform entry point; build.rs never links Apple frameworks there.

## Commands and presentation

`src/cli.rs` defines parsing, command discovery, help, and the searchable menu. The menu builds argument arrays and calls the
same dispatcher as direct CLI commands. It never evaluates a shell command.
src/output.rs handles ASCII bars, width-aware text, control-character escaping,
plain output, and schema-versioned JSON envelopes.

## macOS backends

The Rust dispatcher routes native operations through a short-lived copy of the
same executable. Requests travel as JSON on stdin, responses as JSON on stdout.
The internal __bridge entry point is an implementation detail, not a privilege
boundary or stable public interface. Every worker repeats the platform gate.

native/bridge.m owns framework calls and releases their objects under ARC and
explicit Core Foundation ownership. It catches Objective-C exceptions and emits
error codes and messages. Private frameworks are dynamically resolved.
The worker deadline is 75 seconds, preventing blocked native calls from hanging
the parent indefinitely. EventKit permission waits have a 60-second deadline.

Keyboard writes are read back in a new worker after the writing process exits.
The lid state is checked before a keyboard write. Numeric JSON percentages retain read-back precision.

Subprocesses have fixed executable paths, separate arguments, English locale,
bounded captured output, and deadlines. Child process groups are cleaned up on
timeout. Native requests, including Wi-Fi credentials, are passed via stdin and
are not logged. sudo authentication is handled interactively by macOS.

## Settings transactions

The catalog restricts settings to known keys and value types. Profiles contain
only selected entries. An absent value is represented explicitly with exists=false.
Validation happens for the entire transaction before any preference write.

Transactions lock their state directory, capture previous values, and durably
write a pending journal before the first mutation. Each preference is read back.
Only the required processes are restarted, once each. On failure, prior values
are restored in reverse order, with rollback errors retained in the journal.
Undo detects values changed outside the operation and refuses to overwrite them.

Journal files and locks are private to the user. Values can include user-selected
paths, so journals and profiles should not be published without review.

## Compatibility

Feature availability is determined at runtime. A successful API return is not
sufficient for level setters: read-back must match. Preference read-back verifies
storage; visual application behavior still requires device testing.

Screen resolution changes, app launches, permission-dependent integrations,
network transitions, and administrator commands need separate manual testing.
Compilation on one macOS version does not certify behavior on another.

## System playback

Top-level play/pause commands use distinct MediaRemote command IDs (0 and 1),
resolved dynamically in the isolated native worker. They do not send a toggle
media key or automate a specific app. Native input validation rejects arguments
for either operation. Missing API symbols and rejected submissions return errors.
The output reports a request, because system playback state cannot be reliably
read from an ordinary binary on every supported macOS release.

API reference: [Albert media remote](https://github.com/albertlauncher/albert-plugin-mediaremote).
