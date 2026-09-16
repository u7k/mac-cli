#!/usr/bin/env python3
"""Generate a reference from the same command tree used by the CLI and menu."""
import json
import pathlib
import subprocess
import sys

root=pathlib.Path(__file__).resolve().parent.parent
check = "--check" in sys.argv
arguments = [a for a in sys.argv[1:] if a != "--check"]
binary=root / (arguments[0] if arguments else "target/debug/mac")
result=subprocess.run([str(binary),"commands","--json"],check=True,capture_output=True,text=True)
commands=json.loads(result.stdout)["data"]
lines=["# Command reference","","Generated from the CLI command tree. All commands support --help.","",
       "Global options: --json, --plain, --yes, --experimental.",""]
for item in commands:
    command=item["command"]
    help_result=subprocess.run([str(binary),*command.split()[1:],"--help"],check=True,capture_output=True,text=True)
    lines.extend(["## "+command,"",item["description"],"","    "+help_result.stdout.split("Usage: ",1)[1].splitlines()[0],"",
                  "Requirements: "+item["requirements"]+".",""])
(root/"docs").mkdir(exist_ok=True)
rendered = "\n".join(lines)
destination = root/"docs/commands.md"
if check:
    if not destination.exists() or destination.read_text(encoding="utf-8") != rendered:
        sys.exit("error: Command documentation is stale. Run python3 scripts/generate-docs.py.")
else:
    destination.write_text(rendered,encoding="utf-8")
print(f"Generated {len(commands)} command entries.")
