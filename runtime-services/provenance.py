#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Record actual selected dependencies, without declaring image admission."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

PIN = "2ed19d64ec3359143409a4b0a9758b4aeef3ee7d"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args])


def collect(core, root, out, mode):
    if mode not in ("0", "1"):
        raise ValueError("unsupported caller mode")
    if git(core, "rev-parse", "HEAD").decode().strip() != PIN:
        raise ValueError("wrong core source")
    dependencies = set()
    files = list(out.glob("*.d"))
    if len(files) != 10:
        raise ValueError("missing compiler dependency rules")
    for file in files:
        rule = file.read_text().replace("\\\n", " ")
        target, sep, inputs = rule.partition(":")
        if not sep or not target.strip() or not inputs.strip():
            raise ValueError("incomplete dependency rule")
        dependencies.update(Path(p).resolve() for p in inputs.split())
    rows = []
    for path in sorted(dependencies):
        data = path.read_bytes()
        if path.is_relative_to(core):
            relative = path.relative_to(core).as_posix()
            original = git(core, "show", PIN + ":" + relative)
            if relative != "source/core/CodalFiber.cpp" and original != data:
                raise ValueError("changed unselected core input")
            if not (relative.startswith("inc/") or relative.startswith("source/")):
                raise ValueError("unsupported core input")
            row = {"origin": "codal-core", "pin": PIN, "path": relative,
                   "original_sha256": digest(original), "compiled_sha256": digest(data)}
        elif path.is_relative_to(root):
            relative = path.relative_to(root).as_posix()
            if not (relative.startswith("runtime-services/") or relative in
                    ("simulation/include/InterruptMask.h", "paired-irq/include/PairedIrqState.h")):
                raise ValueError("unexpected own-source dependency")
            original = git(root, "show", "HEAD:" + relative)
            if original != data:
                raise ValueError("changed own source")
            row = {"origin": "codal-samd", "path": relative, "sha256": digest(data)}
        else:
            raise ValueError("unknown or vendor/toolchain header requires explicit review")
        rows.append(row)
    # Assembly/linker inputs are explicit; GCC .d rules do not cover them.
    for relative in ("runtime-services/startup.S", "runtime-services/control.ld",
                     "asm/CortexContextSwitch.s", "runtime-services/fiber-guards.patch"):
        data = (root / relative).read_bytes()
        if git(root, "show", "HEAD:" + relative) != data:
            raise ValueError("changed assembly/linker/guard input")
        rows.append({"origin": "codal-samd", "path": relative, "sha256": digest(data)})
    library = Path((out / "libgcc-path.txt").read_text().strip())
    result = {"schema": 1, "kind": "prospective-real-services-build",
              "runtime_admission": False,
              "caller_mode": int(mode), "cpu_hz": 120000000,
              "source": git(root, "rev-parse", "HEAD").decode().strip(),
              "core_source": PIN, "inputs": rows,
              "actual_core_patch_sha256": digest((out / "actual-core-patch.diff").read_bytes()),
              "elf_sha256": digest((out / "control.elf").read_bytes()),
              "map_sha256": digest((out / "control.map").read_bytes()),
              "compiler_sha256": digest((out / "compiler.txt").read_bytes()),
              "libgcc_archive_sha256": digest(library.read_bytes()),
              "open": ["Actual guest qualification in both engine profiles",
                       "Complete allocated-member/header/inline and toolchain notice review",
                       "Native Arcade board API/build and consumer admission"]}
    return result


if __name__ == "__main__":
    core, root, out = (Path(p).resolve() for p in sys.argv[1:4])
    result = collect(core, root, out, sys.argv[4])
    (out / "build-provenance.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print("PROSPECTIVE_REAL_SERVICES_BUILD", len(result["inputs"]), result["elf_sha256"])
