#!/usr/bin/env python3
"""Opt-in hardware write test. Captures exact values and restores them in finally."""
import argparse
import json
import subprocess
import sys
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--binary", default="target/debug/mac")
parser.add_argument("--allow-changes", action="store_true", help="Temporarily adjust levels and restore them")
options = parser.parse_args()
if not options.allow_changes:
    parser.error("--allow-changes is required; this is not an automatic test")

def command(args, request=None):
    result = subprocess.run([options.binary, *args], input=json.dumps(request) if request else None,
                            text=True, capture_output=True, timeout=90)
    try:
        response = json.loads(result.stdout)
    except ValueError:
        raise RuntimeError(result.stderr.strip() or "Invalid JSON response")
    if "error" in response:
        raise RuntimeError(json.dumps(response["error"]))
    return response["data"]

failures = []
for name, args, operation in [
    ("volume", ["volume"], "audio.level"),
    ("keyboard", ["keyboard", "brightness"], "keyboard.brightness"),
    ("display", ["brightness"], "brightness"),
]:
    try:
        before = command([*args, "--json"])
        original = before["percent"]
        if before.get("writable") is False:
            print(f"SKIP {name}: lid is closed or hardware is not writable")
            continue
        if original is None:
            print(f"SKIP {name}: no writable percentage")
            continue
    except Exception as error:
        print(f"SKIP {name}: {error}")
        continue
    restore_args = {"action": "set", "value": original, "input": False}
    if name == "display":
        restore_args["display"] = before["display"]
    target = min(100, int(original) + 5) if original < 95 else max(0, int(original) - 5)
    try:
        response = command([*args, "set", str(target), "--json"])
        time.sleep(0.5)
        observed = command([*args, "--json"])
        if abs(observed["percent"] - target) > 2:
            raise RuntimeError("Value did not persist after command exit")
        print(f"PASS {name}: requested value persisted")
    except Exception as error:
        failures.append(name)
        print(f"FAIL {name}: {error}")
    finally:
        try:
            command(["__bridge"], {"op": operation, "args": restore_args})
            time.sleep(0.2)
            restored = command([*args, "--json"])
            if abs(restored["percent"] - original) > 0.1:
                raise RuntimeError("Read-back differs from the original value")
            print(f"RESTORED {name}")
        except Exception as error:
            failures.append(name + " restore")
            print(f"RESTORE FAILED {name}: {error}", file=sys.stderr)
sys.exit(bool(failures))
