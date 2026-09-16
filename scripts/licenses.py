#!/usr/bin/env python3
"""Bundle dependency license texts without disclosing local build paths."""
import json
import pathlib
import shutil
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parent.parent
output = pathlib.Path(sys.argv[1])
output.mkdir(parents=True, exist_ok=False)
metadata = json.loads(subprocess.check_output(
    ["cargo", "metadata", "--locked", "--offline", "--format-version", "1", "--filter-platform", "aarch64-apple-darwin"], cwd=root))
nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
reachable = set()
pending = [metadata["resolve"]["root"]]
while pending:
    identifier = pending.pop()
    if identifier in reachable:
        continue
    reachable.add(identifier)
    pending.extend(dep["pkg"] for dep in nodes[identifier]["deps"])
index = []
for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
    if package["source"] is None or package["id"] not in reachable:
        continue
    base = pathlib.Path(package["manifest_path"]).parent
    candidates = [p for p in base.iterdir() if p.is_file() and not p.is_symlink()
                  and p.name.lower().startswith(("license", "copying", "copyright", "notice"))]
    if package.get("license_file"):
        declared = base / package["license_file"]
        if not declared.resolve().is_relative_to(base.resolve()):
            sys.exit("error: A dependency license path escapes its package.")
        if declared.is_file() and not declared.is_symlink():
            candidates.append(declared)
    if not candidates:
        sys.exit("error: No license text found for " + package["name"])
    destination = output / (package["name"] + "-" + package["version"])
    destination.mkdir()
    for path in set(candidates):
        shutil.copyfile(path, destination / path.name)
    index.append({"name": package["name"], "version": package["version"],
                  "license": package["license"]})
sysroot = pathlib.Path(subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip())
rust = sysroot / "share/doc/rust"
shutil.copytree(rust / "licenses", output / "licenses")
shutil.copyfile(rust / "COPYRIGHT-library.html", output / "rust-standard-library-copyright.html")
(output / "index.json").write_text(json.dumps(index, indent=2) + "\n", encoding="utf-8")
print("Bundled license notices for", len(index), "dependencies and the Rust standard library.")
