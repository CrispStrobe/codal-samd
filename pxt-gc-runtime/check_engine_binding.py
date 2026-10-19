#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Admit the explicit engine-only substitutions and immutable historical census."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def main():
    manifest = json.loads((ROOT / "profiles/pxt-fixed-engine-adoption.json").read_text())
    assert manifest["schema"] == "fixed-engine-binding.v1"
    assert manifest["engine"] == "79071f69cd6035a820cbdad93da7343d7d501e4d"
    assert len(manifest["bindings"]) == 33
    assert sum(row["replacements"] for row in manifest["bindings"]) == 55
    paths = [row["path"] for row in manifest["bindings"]]
    assert paths == sorted(set(paths))
    for row in manifest["bindings"]:
        raw = (ROOT / row["path"]).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == row["after_sha256"], row["path"]
        assert manifest["old_engine"].encode() not in raw, row["path"]
        assert raw.count(manifest["engine"].encode()) == row["replacements"], row["path"]
    for path, digest in manifest["immutable_historical"].items():
        assert hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest, path
    print("FIXED_ENGINE_BINDING_PASS files=33 substitutions=55 historical=2")

if __name__ == "__main__":
    main()
