# SPDX-License-Identifier: MIT
"""Closed authored source inventory; never a compiler or admission gate."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

SOURCE = '2a991760339fe89f5272f923baf89dc18fd434d0'
BASE = '5bd6b93c219c7e784e885ba2d6812809fb6289a8'
PREFIX = 'simulation/'
FIXTURES = {
    'abstract-transfer-state': ['arm/probe.cpp', 'arm/startup.S', 'arm/control.ld',
                                'include/SpiTransferState.h'],
    'primask-state': ['arm/interrupt-mask-probe.cpp', 'arm/startup.S', 'arm/control.ld',
                      'include/InterruptMask.h'],
    'software-pended-irq0': ['arm/irq-mask-probe.cpp', 'arm/irq-startup.S', 'arm/control.ld',
                            'include/InterruptMask.h'],
}
FIXTURES = {name: [PREFIX + p for p in paths] for name, paths in FIXTURES.items()}
FILES = frozenset(p for paths in FIXTURES.values() for p in paths)


def closed_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError('duplicate JSON key: ' + key)
        result[key] = value
    return result


def validate(record, current_bytes, pinned_bytes, required=()):
    if type(record) is not dict or set(record) != {
            'schema', 'name', 'kind', 'source', 'historical_base', 'runtime_admission',
            'runtime_build_selection', 'files', 'fixtures'}:
        raise ValueError('unknown or missing profile field')
    if type(record['schema']) is not int or record['schema'] != 1:
        raise ValueError('unsupported schema')
    if record['name'] != 'armv7em-authored-foundations' or record['kind'] != 'source-inventory-only':
        raise ValueError('not the reviewed foundation inventory')
    if record['source'] != SOURCE or record['historical_base'] != BASE:
        raise ValueError('source/base adoption requires a separate reviewed change')
    if record['runtime_admission'] is not False or record['runtime_build_selection'] is not False:
        raise ValueError('inventory cannot admit or select a runtime')
    if record['fixtures'] != FIXTURES:
        raise ValueError('unknown fixture, consumer or requested service')
    if type(record['files']) is not dict or set(record['files']) != FILES:
        raise ValueError('unknown/missing source or header; no ASF fallback')
    if any(name not in FIXTURES for name in required):
        raise ValueError('requested runtime/peripheral service is not implemented by this inventory')
    for path in sorted(FILES):
        original = pinned_bytes(path)
        actual = current_bytes(path)
        if actual != original or record['files'][path] != hashlib.sha256(original).hexdigest():
            raise ValueError('changed or unbound selected bytes: ' + path)
    return {'status': 'SOURCE_PROFILE_ONLY', 'source': SOURCE,
            'files': len(FILES), 'runtime_admission': False, 'runtime_build_selection': False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--require', action='append', default=[], help='required authored fixture, not a runtime service')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    try:
        record = json.loads((root / 'profiles/armv7em-foundations.json').read_text(),
                            object_pairs_hook=closed_object)

        def current(path):
            target = root / path
            if target.is_symlink() or not target.is_file() or not target.resolve().is_relative_to(root):
                raise ValueError('selected input must be an in-tree regular file: ' + path)
            return target.read_bytes()

        def pinned(path):
            return subprocess.check_output(['git', '-C', str(root), 'show', SOURCE + ':' + path])

        print(json.dumps(validate(record, current, pinned, args.require), sort_keys=True))
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(2, 'foundation profile refused: ' + str(error) + '\n')


if __name__ == '__main__':
    main()
