# Security

Run mac as your normal desktop user. Running the complete program as root is
rejected; individual administrator operations use the system authentication
prompt. Do not install it setuid or grant blanket passwordless sudo access.

The CLI acts with the permissions of its user. It is not a sandbox for hostile
programs already running as that same user. Its private native worker is not an
authentication boundary. macOS permissions remain authoritative.

External commands receive argument arrays, not interpolated shell source.
Native requests are validated before entering Objective-C. Wi-Fi credentials
travel over stdin and are not added to process arguments or stored in journals.
Terminal output escapes control characters and bidirectional formatting marks;
JSON preserves user data through standard JSON encoding.

Settings use a typed catalog, validated snapshots, private descriptor-relative
journals, and rollback. Profiles are configuration inputs: review their diff
before applying files from others. Screenshots never replace an existing
output path, even if that path appears during capture. Cancellation does not
publish a file or change the clipboard.

See docs/security-audit.md for the scope and limitations of the latest review.
A clean dependency advisory scan does not prove the absence of vulnerabilities.

## Reporting

Report vulnerabilities privately through [GitHub Security Advisories](https://github.com/u7k/mac-cli/security/advisories/new).
Do not post passwords, private application data, or exploitable details in a
public issue. Include the affected version, reproducible steps, expected
behavior, impact, and sanitized diagnostic output.
