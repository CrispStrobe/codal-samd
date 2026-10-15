#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Hosted boundary controls for the exact selected byte loop, not ARM execution."""
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'consumer-build'))
import build as consumer


def main():
    consumer.hosted()
    out = ROOT / 'gc-link-evidence'
    selected = (out / 'target-native.cpp').read_text()
    start = 'extern "C" __attribute__((optimize("O2", "no-tree-loop-distribute-patterns"))) void *memset('
    end = 'extern "C" void *memcpy('
    if selected.count(start) != 1 or selected.count(end) != 1:
        raise ValueError('exact selected memset function required')
    function = selected[selected.index(start):selected.index(end)]
    original = (ROOT / 'runtime-services/target.cpp').read_text()
    before = 'extern "C" void *memset('
    if function.replace(start, before) != original[original.index(before):original.index(end)]:
        raise ValueError('original byte loop must be unchanged')
    # Use the actual symbol through a volatile pointer so compiler builtins and
    # inlining at the test call cannot replace the selected implementation.
    harness = r'''
typedef __SIZE_TYPE__ size_t;
FUNCTION
static unsigned char memory[384];
int main() {
    void *(*volatile operation)(void *, int, size_t) = memset;
    const int values[] = {-1, 0, 1, 128, 0x123};
    for (unsigned offset = 0; offset < 32; ++offset)
        for (unsigned size = 0; size <= 257; ++size)
            for (unsigned value = 0; value < 5; ++value) {
                for (unsigned i = 0; i < sizeof(memory); ++i) memory[i] = 0x5a;
                unsigned start = 32 + offset;
                if (operation(memory + start, values[value], size) != memory + start) return 1;
                for (unsigned i = 0; i < sizeof(memory); ++i) {
                    unsigned char expected = i >= start && i < start + size
                        ? static_cast<unsigned char>(values[value]) : 0x5a;
                    if (memory[i] != expected) return 2;
                }
            }
    return 0;
}
'''.replace('FUNCTION', function)
    source = out / 'memset-boundaries.cpp'
    binary = out / 'memset-boundaries-host'
    source.write_text(harness)
    commands = [
        ['g++', '-std=c++11', '-O2', '-ffreestanding', '-fno-builtin',
         '-Wall', '-Wextra', '-Werror', str(source), '-o', str(binary)],
        [str(binary)]]
    for index, command in enumerate(commands):
        result = subprocess.run(command, capture_output=True, timeout=30)
        (out / f'memset-boundaries-{index}.stdout').write_bytes(result.stdout)
        (out / f'memset-boundaries-{index}.stderr').write_bytes(result.stderr)
        if result.returncode != 0:
            raise ValueError(f'actual host boundary control failed: {index}/{result.returncode}')
    binary.unlink()
    print(json.dumps(dict(status='PASS_HOST_SELECTED_MEMSET_BOUNDARIES_ONLY',
                          cases=32 * 258 * 5, arm_guest_qualified=False)))


if __name__ == '__main__':
    main()
