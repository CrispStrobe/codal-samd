#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Hosted compile-only actual NVM caller; never links, executes or admits an image."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

PIN = '2ed19d64ec3359143409a4b0a9758b4aeef3ee7d'
OLD_PIN = '312ae57e0b31f5b9df07a81e9d846945828e3c5a'
SOURCE = 'source/drivers/KeyValueStorage.cpp'
ROOT = Path(__file__).resolve().parents[1]
BOOL_SHA = '68b898fc39d5d2a450df9b7263d984e7d2ad4c02675f4043a62e762f169e4b85'


def need(condition, reason):
    if not condition:
        raise ValueError(reason)


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args])


def digest(data):
    return hashlib.sha256(data).hexdigest()


def hosted():
    need(os.environ.get('GITHUB_ACTIONS') == 'true', 'compile entry is hosted CI only')


def dependencies(text):
    # Owned output/checkout paths must not contain spaces; reject rather than
    # incorrectly interpreting Make escapes or multiple rules as dependencies.
    logical = text.replace('\\\n', ' ')
    target, sep, inputs = logical.partition(':')
    need(sep and target.strip() and inputs.strip(), 'incomplete dependency rule')
    need(':' not in inputs and '\\' not in inputs, 'unsupported dependency syntax')
    return sorted(set(inputs.split()))


def origin(path, core):
    path = path.resolve()
    if path.is_relative_to(core):
        relative = path.relative_to(core).as_posix()
        need(relative.startswith(('inc/', 'source/')), 'unsupported core input')
        return 'codal-core', PIN, relative
    if path.is_relative_to(ROOT):
        relative = path.relative_to(ROOT).as_posix()
        need(relative in {'consumer-build/platform_includes.h', 'consumer-build/nvm-api.cpp',
                          'inc/SAMDNVM.h', 'runtime-services/platform_includes.h',
                          'runtime-services/reviewed-toolchain/stdbool.h'},
             'unselected own/vendor input')
        return 'codal-samd', git(ROOT, 'rev-parse', 'HEAD').decode().strip(), relative
    raise ValueError('external/compiler/vendor header forbidden in this compile-only profile')


def run(core, old_core, out):
    need(not out.exists(), 'output must not exist')
    for path in [core, old_core, out]:
        need(path.is_relative_to(ROOT) and path != ROOT and ' ' not in str(path),
             'owned in-checkout paths required')

    def relative(path):
        return path.relative_to(ROOT).as_posix()

    for checkout, pin in [(core, PIN), (old_core, OLD_PIN)]:
        need(git(checkout, 'rev-parse', 'HEAD').decode().strip() == pin, 'wrong core pin')
        need(not git(checkout, 'status', '--porcelain', '--untracked-files=no'), 'dirty core')
    need(not git(ROOT, 'status', '--porcelain', '--untracked-files=no'), 'dirty runtime')
    reviewed = ROOT / 'runtime-services/reviewed-toolchain'
    need(digest((reviewed / 'stdbool.h').read_bytes()) == BOOL_SHA, 'reviewed GCC header')
    out.mkdir()
    flags = ['-mcpu=cortex-m4', '-mthumb', '-std=c++11', '-O0', '-ffreestanding',
             '-fno-exceptions', '-fno-rtti', '-fno-unwind-tables', '-fno-builtin',
             '-ffunction-sections', '-fdata-sections', '-nostdinc', '-nostdinc++',
             '-DBW_SIMULATION_CODAL_SERVICES=1',
             '-DBW_SIMULATION_NVM_CONSUMER_COMPILE=1',
             '-I', 'consumer-build', '-I', 'inc', '-I', relative(reviewed)]

    def includes(checkout):
        result = []
        for directory in ['core', 'types', 'driver-models', 'drivers']:
            result += ['-I', relative(checkout / 'inc' / directory)]
        return result

    def compile_one(name, source, checkout, selected_flags):
        command = ['arm-none-eabi-g++', *selected_flags, *includes(checkout),
                   '-include', 'inc/SAMDNVM.h', '-MD',
                   '-MF', relative(out / (name + '.d')), '-c', relative(source),
                   '-o', relative(out / (name + '.o'))]
        result = subprocess.run(command, capture_output=True, cwd=ROOT)
        (out / (name + '.stderr')).write_bytes(result.stderr)
        return result

    # Real historical API mismatch, not an arbitrary failed compiler command.
    negative = compile_one('historical-api-refusal', old_core / SOURCE, old_core, flags)
    need(negative.returncode != 0 and b'conflicting return type' in negative.stderr
         and b'SAMDNVM::getFlashEnd' in negative.stderr, 'historical API did not refuse specifically')
    missing = compile_one('missing-selector-refusal', core / SOURCE, core,
                          [x for x in flags if x != '-DBW_SIMULATION_NVM_CONSUMER_COMPILE=1'])
    need(missing.returncode != 0 and
         b'Select the NVM consumer compile-only profile explicitly' in missing.stderr,
         'missing selector did not refuse specifically')
    for name, source in [('KeyValueStorage', core / SOURCE),
                         ('nvm-api', ROOT / 'consumer-build/nvm-api.cpp')]:
        result = compile_one(name, source, core, flags)
        if result.returncode:
            # Diagnostic source locations are public checkout locations only;
            # no operator host information is passed to this hosted-only entry.
            print(result.stderr.decode(errors='replace'))
            raise ValueError('actual consumer compilation failed: ' + name)

    rows, used = [], set()
    for name in ['KeyValueStorage', 'nvm-api']:
        for dependency in dependencies((out / (name + '.d')).read_text()):
            path = (ROOT / dependency).resolve()
            source_origin, pin, relative = origin(path, core)
            data = path.read_bytes()
            checkout = core if source_origin == 'codal-core' else ROOT
            need(data == git(checkout, 'show', pin + ':' + relative), 'changed selected input')
            key = (source_origin, relative)
            if key in used:
                continue
            used.add(key)
            rows.append({'origin': source_origin, 'pin': pin, 'path': relative,
                         'sha256': digest(data)})
            destination = out / 'selected-sources' / source_origin / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
    need(('codal-core', SOURCE) in used and ('codal-samd', 'inc/SAMDNVM.h') in used,
         'actual caller/header missing')
    notices = out / 'notices'
    notices.mkdir()
    for name, source in [('LICENSE-codal-core', core / 'LICENSE'),
                         ('LICENSE-codal-samd', ROOT / 'LICENSE'),
                         ('COPYING3', reviewed / 'COPYING3'),
                         ('COPYING.RUNTIME', reviewed / 'COPYING.RUNTIME')]:
        shutil.copyfile(source, notices / name)
    symbols = subprocess.check_output(['arm-none-eabi-nm', '-C', '-u',
                                       str(out / 'KeyValueStorage.o')]).decode()
    (out / 'undefined-symbols.txt').write_text(symbols)
    need('strlen' in symbols and 'strcmp' in symbols, 'libc unexpectedly implemented/linked')
    report = {'schema': 1, 'status': 'ACTUAL_NVM_CONSUMER_COMPILE_ONLY',
              'runtime_source': git(ROOT, 'rev-parse', 'HEAD').decode().strip(),
              'core_source': PIN, 'negative_core_source': OLD_PIN,
              'source_inputs': sorted(rows, key=lambda row: (row['origin'], row['path'])),
              'objects': {name: digest((out / (name + '.o')).read_bytes())
                          for name in ['KeyValueStorage', 'nvm-api']},
              'compiler': subprocess.check_output(['arm-none-eabi-g++', '--version']).decode(),
              'historical_api_refused': True, 'missing_selector_refused': True,
              'linked': False, 'executed': False, 'image_admitted': False,
              'storage_model_qualified': False, 'licence_clearance': False,
              'boundary': 'Real current-core KeyValueStorage and SAMDNVM declaration compatibility '
              'only. Not original PXT ZFlash/RAFFS/settings compilation, a SAMD51 NVM '
              'implementation, whole-board ABI closure, linked firmware, guest or active RTx.'}
    (out / 'compile-report.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    # Artifacts retain source/notices/metadata, not objects or a runnable image.
    for name in ['KeyValueStorage', 'nvm-api']:
        (out / (name + '.o')).unlink()
    print('ACTUAL_NVM_CONSUMER_COMPILE_ONLY PASS; no link, guest, flash or image admission')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('core', type=Path)
    parser.add_argument('historical_core', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    hosted()
    run(args.core.resolve(), args.historical_core.resolve(), args.output.resolve())
