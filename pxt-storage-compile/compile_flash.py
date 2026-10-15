#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Hosted compile of an explicitly include-adapted real PXT SAMD51 flash driver."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools/pxt-storage'))
import sources
sys.path.insert(0, str(ROOT / 'consumer-build'))
import build as consumer

CORE = consumer.PIN
DRIVER = 'pxtapp/settings/SAMDFlash.cpp'
FLASH = 'pxtapp/settings/Flash.h'
ADAPT_FROM = b'#include "pxt.h"\n'
ADAPT_TO = b'#include "pxt-flash-platform.h"\n'
OWN = {'pxt-storage-compile/platform_includes.h',
       'pxt-storage-compile/pxt-flash-platform.h',
       'runtime-services/platform_includes.h',
       'runtime-services/reviewed-toolchain/stdbool.h'}


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def adapt(raw):
    sources.need(raw.startswith(ADAPT_FROM) and raw.count(ADAPT_FROM) == 1,
                 'expected unique original PXT include')
    return ADAPT_TO + raw[len(ADAPT_FROM):]


def origin(path, core, out):
    path = path.resolve()
    if path == out / 'SAMDFlash-storage.cpp':
        return 'adapted-driver', DRIVER
    if path == out / 'Flash.h':
        return 'pxt-common-packages', FLASH
    if path.is_relative_to(core):
        name = path.relative_to(core).as_posix()
        sources.need(name.startswith('inc/'), 'unselected core dependency')
        return 'codal-core', name
    if path.is_relative_to(ROOT):
        name = path.relative_to(ROOT).as_posix()
        sources.need(name in OWN, 'unknown own/vendor dependency')
        return 'codal-samd', name
    raise ValueError('external/compiler/vendor dependency forbidden')


def run(core, framework, out):
    consumer.hosted()
    for path in [core, framework, out]:
        sources.need(path.is_relative_to(ROOT) and path != ROOT and ' ' not in str(path),
                     'owned in-checkout paths required')
    sources.need(not out.exists() and not out.is_symlink(), 'output must be new')
    sources.need(consumer.git(core, 'rev-parse', 'HEAD').decode().strip() == CORE and
                 not consumer.git(core, 'status', '--porcelain', '--untracked-files=no'),
                 'exact clean core required')
    sources.need(not consumer.git(ROOT, 'status', '--porcelain', '--untracked-files=no'),
                 'dirty runtime')
    profile = sources.load(ROOT / 'profiles/pxt-framework-sources.json')
    payloads, notice = sources.verify(profile, framework)
    original, flash = payloads[DRIVER], payloads[FLASH]
    adapted = adapt(original)
    reviewed = ROOT / 'runtime-services/reviewed-toolchain'
    sources.need(digest((reviewed / 'stdbool.h').read_bytes()) == consumer.BOOL_SHA,
                 'reviewed compiler header')
    out.mkdir()
    (out / 'SAMDFlash-original.cpp').write_bytes(original)
    (out / 'SAMDFlash-storage.cpp').write_bytes(adapted)
    (out / 'Flash.h').write_bytes(flash)
    flags = ['-mcpu=cortex-m4', '-mthumb', '-std=c++11', '-O0', '-ffreestanding',
             '-fno-exceptions', '-fno-rtti', '-fno-unwind-tables', '-fno-builtin',
             '-ffunction-sections', '-fdata-sections', '-nostdinc', '-nostdinc++',
             '-DBW_SIMULATION_CODAL_SERVICES=1', '-DBW_PXT_FLASH_COMPILE=1', '-DSAMD51=1']
    for directory in [ROOT / 'pxt-storage-compile', out, reviewed,
                      *[core / 'inc' / part for part in
                        ['core', 'types', 'driver-models', 'drivers']]]:
        flags += ['-I', directory.relative_to(ROOT).as_posix()]

    def compile_one(name, source, selected):
        command = ['arm-none-eabi-g++', *selected, '-MD', '-MF',
                   (out / (name + '.d')).relative_to(ROOT).as_posix(), '-c',
                   source.relative_to(ROOT).as_posix(), '-o',
                   (out / (name + '.o')).relative_to(ROOT).as_posix()]
        result = subprocess.run(command, cwd=ROOT, capture_output=True)
        (out / (name + '.stderr')).write_bytes(result.stderr)
        return result

    missing = compile_one('missing-selector', out / 'SAMDFlash-storage.cpp',
                          [f for f in flags if f != '-DBW_PXT_FLASH_COMPILE=1'])
    sources.need(missing.returncode != 0 and
                 b'Select the SAMD51 PXT flash compile-only profile explicitly' in missing.stderr,
                 'missing profile must refuse specifically')
    unadapted = compile_one('unadapted-driver', out / 'SAMDFlash-original.cpp', flags)
    sources.need(unadapted.returncode != 0 and b'pxt.h: No such file or directory' in unadapted.stderr,
                 'original kitchen-sink include must not silently resolve to a stub')
    result = compile_one('SAMDFlash-storage', out / 'SAMDFlash-storage.cpp', flags)
    if result.returncode:
        print(result.stderr.decode(errors='replace'))
        raise ValueError('actual include-adapted PXT flash driver compilation failed')
    source_pin = consumer.git(ROOT, 'rev-parse', 'HEAD').decode().strip()
    rows = []
    for item in consumer.dependencies((out / 'SAMDFlash-storage.d').read_text()):
        path = (ROOT / item).resolve()
        component, name = origin(path, core, out)
        raw = path.read_bytes()
        if component == 'adapted-driver':
            expected, pin = adapted, sources.PIN
        elif component == 'pxt-common-packages':
            expected, pin = flash, sources.PIN
        else:
            checkout, pin = (core, CORE) if component == 'codal-core' else (ROOT, source_pin)
            expected = consumer.git(checkout, 'show', pin + ':' + name)
        sources.need(raw == expected, 'selected dependency bytes changed')
        rows.append(dict(component=component, pin=pin, path=name, sha256=digest(raw)))
        target = out / 'selected-sources' / component / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)
    sources.need(any(r['component'] == 'adapted-driver' for r in rows) and
                 any(r['component'] == 'pxt-common-packages' for r in rows),
                 'real driver and Flash interface must be selected')
    symbols = subprocess.check_output(['arm-none-eabi-nm', '-C', '-u',
                                      str(out / 'SAMDFlash-storage.o')]).decode()
    for symbol in ['target_panic', 'target_disable_irq', 'target_enable_irq', 'memcpy', 'memset']:
        sources.need(symbol in symbols, 'unresolved real dependency missing: ' + symbol)
    defined = subprocess.check_output(['arm-none-eabi-nm', '-C', '--defined-only',
                                      str(out / 'SAMDFlash-storage.o')]).decode()
    for method in ['totalSize()', 'pageSize(', 'erasePage(', 'writeBytes(']:
        sources.need('codal::ZFlash::' + method in defined, 'real driver method missing')
    (out / 'undefined-symbols.txt').write_text(symbols)
    notices = out / 'notices'
    notices.mkdir()
    (notices / 'LICENSE-pxt-common-packages').write_bytes(notice)
    for name, path in [('LICENSE-codal-core', core / 'LICENSE'),
                       ('LICENSE-codal-samd', ROOT / 'LICENSE'),
                       ('COPYING3', reviewed / 'COPYING3'),
                       ('COPYING.RUNTIME', reviewed / 'COPYING.RUNTIME')]:
        shutil.copyfile(path, notices / name)
    report = dict(schema=1, status='INCLUDE_ADAPTED_REAL_PXT_FLASH_COMPILE_ONLY',
        runtime_source=source_pin, core_source=CORE, framework_source=sources.PIN,
        adaptation=dict(path=DRIVER, original_sha256=digest(original),
                        adapted_sha256=digest(adapted), replaced_include_lines=1),
        source_inputs=rows, compiler=subprocess.check_output(['arm-none-eabi-g++', '--version']).decode(),
        object_sha256=digest((out / 'SAMDFlash-storage.o').read_bytes()),
        missing_selector_refused=True, unadapted_without_pxt_stub_refused=True,
        full_pxt_headers_compiled=False, raffs_settings_compiled=False,
        original_board_configuration_qualified=False, linked=False, executed=False,
        storage_model_qualified=False, image_admitted=False, licence_clearance=False)
    (out / 'compile-report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k not in ['source_inputs', 'compiler']}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('core', type=Path)
    parser.add_argument('framework', type=Path)
    parser.add_argument('out', type=Path)
    args = parser.parse_args()
    run(args.core.resolve(), args.framework.resolve(), args.out.resolve())
