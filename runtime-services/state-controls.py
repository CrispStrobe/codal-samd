#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Hosted host-only runtime state controls, not ARM/driver/guest admission."""
import os
from pathlib import Path
import signal
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
original = (root / "runtime-services/RuntimeSpiTransferState.h").read_text()
control = (root / "simulation/tests/spi-transfer-state.cpp").read_text()
anchors = {
    '#include "SpiTransferState.h"': '#include "RuntimeSpiTransferState.h"',
    "using State = brickwright_simulation::SpiTransferState;":
        "using State = brickwright_runtime::RuntimeSpiTransferState;",
}
for before, after in anchors.items():
    assert control.count(before) == 1, "original control source changed"
    control = control.replace(before, after)
mutations = {
    "premature-success": ("phase_ = Phase::draining;", "finish(Result::success);"),
    "error-is-success": ("fail(token, Result::dma_error)", "fail(token, Result::success)"),
    "stale-token": ("token != 0 && token == generation_", "token != 0"),
    "duplicate-consume": ("result = result_;\n        phase_ = Phase::idle;",
                          "result = result_;\n        phase_ = Phase::terminal;"),
    "overwrite-pending": ("phase_ != Phase::idle || ", ""),
}
with tempfile.TemporaryDirectory(prefix="runtime-spi-state-", dir=os.environ["RUNNER_TEMP"]) as tmp:
    directory = Path(tmp)
    source = directory / "control.cpp"
    source.write_text(control)
    for name in ("positive", *mutations):
        header = original
        if name != "positive":
            before, after = mutations[name]
            assert header.count(before) == 1, "runtime mutation anchor changed"
            header = header.replace(before, after)
        (directory / "RuntimeSpiTransferState.h").write_text(header)
        binary = directory / "control"
        subprocess.run(["g++", "-std=c++11", "-Wall", "-Wextra", "-Werror",
                        "-pedantic", "-UNDEBUG", "-I", str(directory), str(source),
                        "-o", str(binary)], check=True)
        result = subprocess.run([str(binary)], capture_output=True)
        if name == "positive":
            assert result.returncode == 0, "runtime state positive refused"
        else:
            assert result.returncode == -signal.SIGABRT and b"Assertion" in result.stderr, name
        print("RUNTIME_SPI_STATE_CONTROL", name, "PASS", flush=True)
