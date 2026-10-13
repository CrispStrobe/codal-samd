#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Record actual selected dependencies, without declaring image admission."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

PIN = "2ed19d64ec3359143409a4b0a9758b4aeef3ee7d"
GCC_PIN = "c891d8dc23e1a46ad9f3e757d09e57b500d40044"
BOOL_SHA = "68b898fc39d5d2a450df9b7263d984e7d2ad4c02675f4043a62e762f169e4b85"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args])


def collect(core, root, out, mode):
    if mode not in ("0", "1", "2", "3"):
        raise ValueError("unsupported caller mode")
    if git(core, "rev-parse", "HEAD").decode().strip() != PIN:
        raise ValueError("wrong core source")
    library = Path((out / "libgcc-path.txt").read_text().strip()).resolve()
    include_dir = Path((out / "compiler-include-path.txt").read_text().strip()).resolve()
    known_header = (include_dir / "stdbool.h").resolve()
    loaded = [Path(line[5:].strip()).resolve() for line in
              (out / "control.map").read_text().splitlines() if line.startswith("LOAD ")]
    if library not in loaded:
        raise ValueError("recorded libgcc multilib is not the actual map-loaded archive")
    reviewed = root / "runtime-services/reviewed-toolchain/stdbool.h"
    if digest(reviewed.read_bytes()) != BOOL_SHA or git(root, "show", "HEAD:" + reviewed.relative_to(root).as_posix()) != reviewed.read_bytes():
        raise ValueError("reviewed GCC header changed")
    dependencies = set()
    files = list(out.glob("*.d"))
    if len(files) != 11:
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
        elif path == known_header:
            if data != reviewed.read_bytes():
                raise ValueError("installed stdbool.h differs from reviewed immutable GCC source")
            row = {"origin": "gcc-header", "pin": GCC_PIN,
                   "path": "gcc/ginclude/stdbool.h", "sha256": digest(data),
                   "notice": "GPL-3.0-or-later WITH GCC-exception-3.1"}
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


def preserve_notices(core, root, out, result):
    # Preserve the actual selected source/header notices, not an inferred
    # whole-image licence label. These inputs are reviewed Git dependencies;
    # unknown/system headers have already refused in collect().
    records = []
    for row in result["inputs"]:
        relative = row["path"]
        if row["origin"] == "gcc-header":
            include_dir = Path((out / "compiler-include-path.txt").read_text().strip()).resolve()
            data = (include_dir / "stdbool.h").read_bytes()
        else:
            provider = core if row["origin"] == "codal-core" else root
            data = (provider / relative).read_bytes()
        expected = row.get("compiled_sha256", row.get("sha256"))
        if digest(data) != expected:
            raise ValueError("selected source changed before notice preservation")
        target = out / "selected-source" / row["origin"] / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
        records.append({"path": target.relative_to(out).as_posix(), "sha256": digest(data)})
    notices = [("codal-core-MIT.txt", core / "LICENSE"),
               ("codal-samd-MIT.txt", root / "LICENSE"),
               ("compiler-package-copyright.txt", Path("/usr/share/doc/gcc-arm-none-eabi/copyright")),
               ("GPL-3.txt", Path("/usr/share/common-licenses/GPL-3")),
               ("GCC-RUNTIME-EXCEPTION-3.1.txt", root / "runtime-services/reviewed-toolchain/COPYING.RUNTIME")]
    for name, source in notices:
        data = source.read_bytes()  # Missing required notice is a failure.
        target = out / "notices" / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
        records.append({"path": target.relative_to(out).as_posix(), "sha256": digest(data)})
    result["preserved_source_and_notice_files"] = records
    # Packaging texts and source notices do not prove exception applicability
    # or complete allocated-member correspondence. Those remain explicit gates.


if __name__ == "__main__":
    core, root, out = (Path(p).resolve() for p in sys.argv[1:4])
    result = collect(core, root, out, sys.argv[4])
    preserve_notices(core, root, out, result)
    (out / "build-provenance.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print("PROSPECTIVE_REAL_SERVICES_BUILD", len(result["inputs"]), result["elf_sha256"])
