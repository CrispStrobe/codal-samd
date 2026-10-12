#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Bind this diagnostic build's exact source/header inputs, not runtime admission."""
import argparse
import hashlib
import json
from pathlib import Path
import shlex
import shutil
import subprocess

PIN = '2ed19d64ec3359143409a4b0a9758b4aeef3ee7d'
CORE = (
    'source/driver-models/Timer.cpp', 'inc/driver-models/Timer.h',
    'inc/driver-models/LowLevelTimer.h', 'inc/core/CodalConfig.h',
    'inc/core/CodalComponent.h', 'inc/core/ErrorNo.h',
    'inc/core/codal_target_hal.h', 'inc/types/Event.h',
    'inc/core/CodalCompat.h', 'inc/core/CodalDmesg.h', 'LICENSE',
)
SAMD = (
    'timer-caller/probe.cpp', 'timer-caller/platform_includes.h',
    'timer-caller/build.sh', 'timer-caller/provenance.py', 'timer-caller/test_provenance.py',
    'paired-irq/profile.cpp', 'paired-irq/include/PairedIrqState.h',
    'simulation/include/InterruptMask.h', 'simulation/arm/irq-startup.S',
    'simulation/arm/control.ld',
)


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def record(data):
    return {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}


def command(*args):
    return subprocess.check_output(args).decode().strip()


def dependencies(text):
    """One GCC make dependency rule; escapes supported, extra rules refused."""
    logical = text.replace('\\\n', '')
    lines = [line for line in logical.splitlines() if line.strip()]
    require(len(lines) == 1 and ':' in lines[0], 'one complete dependency rule required')
    target, body = lines[0].split(':', 1)
    require(bool(target.strip()), 'missing dependency target')
    names = shlex.split(body)
    require(bool(names) and all(':' not in name for name in names), 'invalid dependency body')
    # GCC can repeat guarded headers reached through different include paths.
    # Preserve every occurrence here so verify checks each binding and byte hash.
    return names


def capture(core, samd):
    core, samd = core.resolve(), samd.resolve()
    require(command('git', '-C', str(core), 'rev-parse', 'HEAD') == PIN, 'core pin mismatch')
    files, paths = {}, {}
    for role, root, names in [('core', core, CORE), ('samd', samd, SAMD)]:
        require(not command('git', '-C', str(root), 'status', '--porcelain', '--untracked-files=no'),
                role + ' tracked tree is dirty')
        for name in names:
            path = root / name
            require(path.resolve().is_relative_to(root), 'source escapes its checkout')
            data = path.read_bytes()
            original = subprocess.check_output(['git', '-C', str(root), 'show', 'HEAD:' + name])
            require(data == original, role + ' source differs from immutable Git')
            label = role + '/' + name
            files[label], paths[path.resolve()] = record(data), label
    compiler = shutil.which('arm-none-eabi-g++')
    require(compiler is not None, 'cross compiler missing')
    driver = Path(compiler).resolve()
    libgcc = Path(command(str(driver), '-print-libgcc-file-name')).resolve()
    include = Path(command(str(driver), '-print-file-name=include')).resolve()
    # The actual Timer closure currently consumes only this compiler header.
    # An additional compiler header must be explicitly reviewed/bound, not skipped.
    for label, path in [('compiler/driver', driver), ('compiler/libgcc.a', libgcc),
                        ('compiler/include/stdbool.h', include / 'stdbool.h')]:
        files[label], paths[path.resolve()] = record(path.read_bytes()), label
    result = {'schema': 1, 'core_commit': PIN,
              'samd_commit': command('git', '-C', str(samd), 'rev-parse', 'HEAD'),
              'files': files,
              'boundary': 'Diagnostic build only; no guest, full toolchain or runtime admission'}
    return result, paths


def verify(out, manifest, paths):
    units = {}
    for unit in ('timer', 'profile', 'probe'):
        labels = []
        for name in dependencies((out / (unit + '.d')).read_text()):
            path = Path(name).resolve()
            require(path in paths, 'unbound compiler dependency')
            label = paths[path]
            require(record(path.read_bytes()) == manifest['files'][label], 'dependency drift')
            labels.append(label)
        require(labels, 'empty unit dependency closure')
        required_source = {'timer': 'core/source/driver-models/Timer.cpp',
                           'profile': 'samd/paired-irq/profile.cpp',
                           'probe': 'samd/timer-caller/probe.cpp'}[unit]
        require(required_source in labels, 'wrong unit source binding')
        units[unit] = sorted(set(labels))
    require(set('core/' + name for name in CORE if name != 'LICENSE')
            <= set(units['timer']), 'actual Timer header closure incomplete')
    return {'schema': 1, 'source': manifest, 'units': units,
            'elf': record((out / 'control.elf').read_bytes()),
            'map': record((out / 'control.map').read_bytes()),
            'guest_executed': False, 'runtime_admitted': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=['before', 'after'])
    parser.add_argument('core', type=Path)
    parser.add_argument('samd', type=Path)
    parser.add_argument('out', type=Path)
    args = parser.parse_args()
    manifest, paths = capture(args.core, args.samd)
    if args.mode == 'before':
        result = manifest
    else:
        require(manifest == json.loads((args.out / 'source-before.json').read_text()),
                'source/compiler drift during build')
        result = verify(args.out, manifest, paths)
    print(json.dumps(result, indent=2, sort_keys=True))
