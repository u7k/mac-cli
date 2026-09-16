# Security review — 2026-09-10

Scope: Rust command dispatch, native request validation, subprocess handling,
terminal output, settings journals/profiles, screenshot publication, installation,
dependency advisories, and release scripts. Review method: source inspection
and local regression tests.

## Findings addressed

| Area | Change |
| --- | --- |
| Screenshot path race | Capture in a private directory, then atomically create the destination without replacement; verify capture before copying to clipboard. |
| Journal filesystem handling | Use directory descriptors, no-follow opens, ownership/permissions checks, single-link files, exclusive locks, and bounded regular-file reads. |
| Malformed undo records | Validate all keys, types, snapshots, duplicates, and status before any preference mutation. |
| Partial settings operation | Roll back if final journal persistence fails; preserve absent preferences and exact previous value types. |
| Native worker inputs | Validate operation names, argument fields/types, IDs, percentages, and preference catalog membership before FFI. |
| Misleading terminal text | Escape terminal controls and bidirectional formatting; sanitize confirmation prompts including line breaks. |
| Subprocess timeout | Bound pipe-drain time and avoid signaling a potentially reused process group after its leader has been reaped. |
| Excessive privilege | Refuse root execution of the full CLI. |
| Formula injection | Reject URL fragments/interpolation, credentials, control characters, and malformed HTTPS URLs. |
| Release automation | Pin checkout by commit, minimize CI permissions, disable persisted credentials, and require Developer ID plus accepted notarization for release mode. |

## Evidence

- Rust: 28 unit tests and 6 CLI integration tests passed locally.
- Release scripts: 3 Python regression tests passed.
- Formatting and Clippy with warnings denied passed.
- Clang static analyzer completed without diagnostics for native/bridge.m.
- cargo-audit 0.22.2: 99 locked dependencies, zero known vulnerabilities and no
  warnings, using RustSec commit b50980aad8b8f14f77e25a97b32dd94bf008b0af
  (database updated 2026-09-09T12:49:52+02:00).
- Regression tests use temporary files or in-memory settings and do not modify
  actual user preferences.

## Remaining release requirements

The local archive is ad-hoc signed. Developer ID signing and Apple notarization
have not been completed. The source publication checks below record subsequent verification.

Physical display/keyboard brightness, permission-granted/denied application
workflows, and macOS 14/15 runtime verification remain subject to the explicit
matrix in [compatibility.md](compatibility.md). Private Apple APIs may change
between macOS releases.

## Source publication checks — 2026-09-16

The source release checks passed: 29 unit tests, 6 CLI integration tests,
3 release-script tests, formatting, Clippy, release build, and generated docs.
The refreshed audit checked 98 locked dependencies with zero known
vulnerabilities or warnings (RustSec e2e640471715167f73e22eaf761f2e547adafeec).
Private vulnerability reporting is enabled at https://github.com/u7k/mac-cli.
Developer ID signing and notarization remain separate binary-release steps.
