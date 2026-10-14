#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Changed Markdown file-link/privacy policy; not a renderer or source admission."""
import argparse
from pathlib import Path
import re
import subprocess
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[2]


def validate(path, text):
    if re.search(r"/(?:mnt|home|tmp|Users)/", text):
        raise ValueError(f"private host path in {path.relative_to(ROOT)}")
    count = 0
    for target in re.findall(r"\]\(([^)\n]+)\)", text):
        target = target.strip()
        if target.startswith("<") and target.endswith(">"):
            target = target[1:-1]
        parsed = urlsplit(target)
        if parsed.scheme:
            if parsed.scheme not in ("https", "http", "mailto"):
                raise ValueError("unsupported public link scheme")
            continue
        if parsed.netloc or parsed.path.startswith("/"):
            raise ValueError("absolute/non-repository link")
        if not parsed.path:
            continue
        destination = (path.parent / unquote(parsed.path)).resolve()
        if not destination.is_relative_to(ROOT) or not destination.is_file():
            raise ValueError(f"missing/out-of-tree relative link in {path.relative_to(ROOT)}")
        count += 1
    return count


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", required=True)
    args = parser.parse_args()
    if not re.fullmatch(r"[0-9a-f]{40}", args.base):
        parser.error("base must be an immutable Git commit")
    subprocess.run(["git", "diff", "--check", args.base, "HEAD"], cwd=ROOT, check=True)
    names = subprocess.check_output(
        ["git", "diff", "--name-only", "--diff-filter=AM", "-z", args.base, "HEAD"], cwd=ROOT
    ).decode().split("\0")
    paths = [ROOT / name for name in names if name.endswith(".md")]
    if not paths:
        raise ValueError("no changed Markdown; documentation gate is not admission by absence")
    links = sum(validate(path, path.read_text(encoding="utf-8")) for path in paths)
    print(f"DOCUMENTATION_POLICY PASS files={len(paths)} relative_files={links}; external links/anchors/rendering not asserted")


if __name__ == "__main__":
    main()
