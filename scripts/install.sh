#!/bin/sh
set -eu

project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
binary="$project_root/target/release/mac"
destination="${1:-$HOME/.local/bin}"

if [ "$(uname -s)" != Darwin ]; then
    echo "error: mac-cli requires Apple macOS." >&2
    exit 78
fi
if [ ! -x "$binary" ]; then
    echo "error: Build first with cargo build --release --locked." >&2
    exit 1
fi
"$binary" --version >/dev/null
if command -v mac >/dev/null 2>&1; then
    echo "error: A mac command already exists on PATH; nothing was installed." >&2
    exit 1
fi
if [ -e "$destination/mac" ] || [ -L "$destination/mac" ]; then
    echo "error: The destination already contains mac; nothing was installed." >&2
    exit 1
fi
mkdir -p "$destination"
temporary=$(mktemp "$destination/.mac-cli.XXXXXX")
trap 'rm -f "$temporary"' EXIT HUP INT TERM
cp "$binary" "$temporary"
chmod 755 "$temporary"
# Hard-link creation is atomic and refuses to replace even a dangling symlink.
ln "$temporary" "$destination/mac"
printf 'Installed %s/mac\nAdd %s to PATH if needed.\n' "$destination" "$destination"
