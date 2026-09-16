#!/usr/bin/env python3
"""Create a Homebrew formula using a real release URL and archive checksum."""
import argparse
import hashlib
import json
import pathlib
import re
import urllib.parse

p=argparse.ArgumentParser(description=__doc__)
p.add_argument("--archive",type=pathlib.Path,required=True)
p.add_argument("--url",required=True)
p.add_argument("--homepage",required=True)
p.add_argument("--version",required=True)
p.add_argument("--output",type=pathlib.Path,default=pathlib.Path("dist/mac-cli.rb"))
a=p.parse_args()
for value in [a.url,a.homepage]:
    parsed = urllib.parse.urlparse(value)
    if (parsed.scheme != "https" or not parsed.hostname or parsed.username is not None
            or "#" in value or any(ord(c) < 33 or ord(c) == 127 for c in value)):
        p.error("Use an HTTPS URL without Ruby interpolation or newline characters")
if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:-[A-Za-z0-9.-]+)?",a.version):
    p.error("Invalid version")
sha=hashlib.sha256(a.archive.read_bytes()).hexdigest()
formula=f'''class MacCli < Formula
  desc "English-only macOS command center"
  homepage {json.dumps(a.homepage)}
  url {json.dumps(a.url)}
  version {json.dumps(a.version)}
  sha256 "{sha}"
  license "MIT"
  depends_on macos: :sonoma
  depends_on arch: :arm64

  def install
    bin.install "mac"
  end

  test do
    assert_match "mac", shell_output("#{{bin}}/mac --version")
  end
end
'''
a.output.parent.mkdir(parents=True,exist_ok=True)
with a.output.open("x",encoding="utf-8") as file:
    file.write(formula)
print(a.output)
