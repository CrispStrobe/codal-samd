# SPDX-License-Identifier: MIT
"""Hosted pure policy mutations: compile errors/timeouts are not detection."""
from pathlib import Path
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
header = (root / 'include/PairedIrqState.h').read_text()
control = root / 'tests/state.cpp'
lines = control.read_text().splitlines()
mutants = {
    'always-enable': ('depth_ == 0 ? outerMask_ : 1', 'depth_ == 0 ? 0u : 1u',
                      'CHECK(final.ok() && final.restoreMask == entry'),
    'early-inner-enable': ('depth_ == 0 ? outerMask_ : 1', 'outerMask_',
                           'CHECK(state.leave(1, thread).restoreMask == 1); // nested restoration control'),
    'cross-owner': ('if (!owner_.equals(owner))', 'if (false)',
                    'CHECK(result.fault == Fault::owner_changed'),
    'no-depth-limit': ('if (depth_ == MaxDepth)', 'if (false)',
                       'CHECK(state.enter(1, thread).fault == Fault::depth_limit)'),
}

with tempfile.TemporaryDirectory(prefix='paired-policy-') as directory:
    destination = Path(directory)
    for name, (old, new, expected_line) in mutants.items():
        matches = [i for i, line in enumerate(lines, 1) if expected_line in line]
        if len(matches) != 1 or header.count(old) != (2 if name == 'cross-owner' else 1):
            raise ValueError(f'{name}: mutation/control source premise changed; '
                             f'control matches={len(matches)}, header matches={header.count(old)}')
        expected_code = matches[0]
        if not 0 < expected_code < 256:
            raise ValueError('control exit must be a nonzero unambiguous line code')
        (destination / 'PairedIrqState.h').write_text(header.replace(old, new))
        executable = destination / name
        subprocess.run(['g++', '-std=c++11', '-Wall', '-Wextra', '-Werror', '-pedantic',
                        '-I', str(destination), str(control), '-o', str(executable)],
                       check=True, timeout=30)
        actual = subprocess.run([str(executable)], check=False, timeout=5).returncode
        if actual != expected_code:
            raise ValueError(f'{name}: expected control exit {expected_code}, '
                             f'observed {actual}; not exact mutation detection')
        print(name + ': exact control refusal PASS')
