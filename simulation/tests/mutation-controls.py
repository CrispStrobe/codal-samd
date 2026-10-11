#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 CrispStrobe contributors
"""Hosted-only compiler controls; originals never modified. No runtime/guest."""
import os
from pathlib import Path
import signal
import subprocess
import tempfile

root = Path(__file__).resolve().parents[2]
original = (root / 'simulation/include/SpiTransferState.h').read_text()
test = root / 'simulation/tests/spi-transfer-state.cpp'
mutations = {
    'premature-success': ('phase_ = Phase::draining;', 'finish(Result::success);'),
    'error-is-success': ('fail(token, Result::dma_error)', 'fail(token, Result::success)'),
    'stale-token': ('token != 0 && token == generation_', 'token != 0'),
    'duplicate-consume': ('result = result_;\n        phase_ = Phase::idle;',
                          'result = result_;\n        phase_ = Phase::terminal;'),
    'overwrite-pending': ('phase_ != Phase::idle || ', ''),
}
with tempfile.TemporaryDirectory(prefix='spi-mutants-', dir=os.environ['RUNNER_TEMP']) as tmp:
    directory = Path(tmp)
    for name, (before, after) in mutations.items():
        assert original.count(before) == 1, (name, 'source anchor changed')
        (directory / 'SpiTransferState.h').write_text(original.replace(before, after))
        binary = directory / 'control'
        subprocess.run(['g++', '-std=c++11', '-Wall', '-Wextra', '-Werror',
                        '-pedantic', '-UNDEBUG', '-I', str(directory), str(test),
                        '-o', str(binary)], check=True)
        result = subprocess.run([str(binary)], capture_output=True)
        assert result.returncode == -signal.SIGABRT, (name, result.returncode)
        assert b'Assertion' in result.stderr, (name, 'not an assertion rejection')
        print(name, 'assertion-detected', flush=True)
