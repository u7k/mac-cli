#!/bin/sh
set -eu
mode="${1:---local}"
case "$mode" in --local|--release) ;; *) echo "Usage: sh scripts/package.sh [--local|--release]" >&2; exit 2 ;; esac
if [ "$mode" = --release ]; then
    if [ -z "${MAC_CLI_SIGN_IDENTITY:-}" ] || [ -z "${MAC_CLI_NOTARY_PROFILE:-}" ]; then
        echo "error: Public releases require a Developer ID Application identity and a notarytool keychain profile." >&2
        exit 1
    fi
fi

project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_root"
if [ "$(uname -s)" != Darwin ] || [ "$(uname -m)" != arm64 ]; then
    echo "error: Package on Apple Silicon macOS." >&2
    exit 78
fi
if [ "$mode" = --release ]; then
    sh scripts/release-check.sh
else
    cargo build --release --locked
fi
version=$(target/release/mac --version | awk '{print $2}')
stage=$(mktemp -d "$project_root/target/package.XXXXXX")
trap 'rm -rf "$stage"' EXIT HUP INT TERM
mkdir -p "$project_root/dist"
cp target/release/mac LICENSE README.md SECURITY.md "$stage/"
cp -R docs "$stage/"
python3 scripts/licenses.py "$stage/third-party-licenses"
if [ -n "${MAC_CLI_SIGN_IDENTITY:-}" ]; then
    codesign --force --sign "$MAC_CLI_SIGN_IDENTITY" --identifier io.uygur.mac-cli --options runtime --timestamp "$stage/mac"
else
    codesign --force --sign - --identifier io.uygur.mac-cli "$stage/mac"
fi
codesign --verify --strict "$stage/mac"
if [ "$mode" = --release ]; then
    details=$(codesign -dv --verbose=4 "$stage/mac" 2>&1)
    case "$details" in
        *"Authority=Developer ID Application:"*) ;;
        *) echo "error: A development or ad-hoc signature cannot be published as a Developer ID release." >&2; exit 1 ;;
    esac
fi
if [ -n "${MAC_CLI_NOTARY_PROFILE:-}" ]; then
    if [ -z "${MAC_CLI_SIGN_IDENTITY:-}" ]; then
        echo "error: Notarization requires MAC_CLI_SIGN_IDENTITY." >&2
        exit 1
    fi
    ditto -c -k --keepParent "$stage/mac" "$stage/notarize.zip"
    xcrun notarytool submit "$stage/notarize.zip" --keychain-profile "$MAC_CLI_NOTARY_PROFILE" --wait --output-format json > "$stage/notary-result.json"
    python3 - "$stage/notary-result.json" <<'PY'
import json, sys
result=json.load(open(sys.argv[1]))
if result.get("status") != "Accepted":
    sys.exit("error: Apple did not accept the notarization submission.")
PY
    cp "$stage/notary-result.json" "dist/mac-cli-$version-notary.json"
    rm "$stage/notarize.zip" "$stage/notary-result.json"
fi
archive="dist/mac-cli-$version-aarch64-apple-darwin.tar.gz"
COPYFILE_DISABLE=1 tar -czf "$archive" -C "$stage" .
shasum -a 256 "$archive" > "$archive.sha256"
printf 'Created %s\n' "$archive"
if [ -z "${MAC_CLI_SIGN_IDENTITY:-}" ]; then
    echo "Local ad-hoc package only; Developer ID signing and notarization have not run."
fi
