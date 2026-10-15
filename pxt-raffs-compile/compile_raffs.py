#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Hosted include-adapted real RAFFS compile, never a filesystem execution test."""
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
RAFFS = 'pxtapp/settings/RAFFS.cpp'
PUBLIC = {'Flash.h': 'pxtapp/settings/Flash.h', 'RAFFS.h': 'pxtapp/settings/RAFFS.h'}
OWN = {'pxt-raffs-compile/platform_includes.h', 'pxt-raffs-compile/pxt-raffs-platform.h',
       'runtime-services/platform_includes.h', 'runtime-services/reviewed-toolchain/stdbool.h'}
DECL = b'uint32_t hash_fnv1(const void *data, unsigned len);'
REPLACEMENTS = [(b'#include "pxtbase.h"\n', b'#include "pxt-raffs-platform.h"\n'),
                (b'#include <stddef.h>\n', b'#include "pxt-raffs-platform.h"\n')]


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def adapt(raw, base):
    for before, after in REPLACEMENTS:
        sources.need(raw.count(before) == 1, 'unique original RAFFS include required')
        raw = raw.replace(before, after, 1)
    sources.need(base.splitlines().count(DECL) == 1, 'exact public hash declaration required')
    sources.need(base.splitlines().count(b'using namespace pxt;') == 1,
                 'exact public namespace exposure required')
    declaration = b'// Extracted from admitted public pxtbase.h; component MIT notice retained.\n'
    declaration += b'#pragma once\nnamespace pxt {\n' + DECL + b'\n}\nusing namespace pxt;\n'
    return raw, declaration


def origin(path, core, out):
    path = path.resolve()
    if path == out / 'RAFFS-storage.cpp':
        return 'adapted-raffs', RAFFS
    if path == out / 'pxt-hash-declaration.h':
        return 'extracted-declaration', 'pxtapp/pxtbase.h'
    for local, public in PUBLIC.items():
        if path == out / local:
            return 'pxt-common-packages', public
    if path.is_relative_to(core):
        relative = path.relative_to(core).as_posix()
        sources.need(relative.startswith('inc/'), 'unselected core input')
        return 'codal-core', relative
    if path.is_relative_to(ROOT):
        relative = path.relative_to(ROOT).as_posix()
        sources.need(relative in OWN, 'unknown own/vendor input')
        return 'codal-samd', relative
    raise ValueError('external/compiler/vendor input forbidden')


def run(core, framework, out):
    consumer.hosted()
    for path in [core, framework, out]:
        sources.need(path.is_relative_to(ROOT) and path != ROOT and ' ' not in str(path),
                     'owned in-checkout paths required')
    sources.need(not out.exists() and not out.is_symlink(), 'output must be new')
    sources.need(consumer.git(core, 'rev-parse', 'HEAD').decode().strip() == CORE and
                 not consumer.git(core, 'status', '--porcelain', '--untracked-files=no'),
                 'clean exact core required')
    sources.need(not consumer.git(ROOT, 'status', '--porcelain', '--untracked-files=no'), 'dirty runtime')
    payloads, notice = sources.verify(sources.load(ROOT / 'profiles/pxt-framework-sources.json'), framework)
    original, base = payloads[RAFFS], payloads['pxtapp/pxtbase.h']
    adapted, declaration = adapt(original, base)
    reviewed = ROOT / 'runtime-services/reviewed-toolchain'
    sources.need(digest((reviewed / 'stdbool.h').read_bytes()) == consumer.BOOL_SHA,
                 'reviewed compiler header')
    out.mkdir()
    files = {'RAFFS-original.cpp': original, 'RAFFS-storage.cpp': adapted,
             'pxtbase-original.h': base, 'pxt-hash-declaration.h': declaration,
             **{name: payloads[path] for name, path in PUBLIC.items()}}
    for name, raw in files.items():
        (out / name).write_bytes(raw)
    flags = ['-mcpu=cortex-m4', '-mthumb', '-std=c++11', '-O0', '-ffreestanding',
             '-fno-exceptions', '-fno-rtti', '-fno-unwind-tables', '-fno-builtin',
             '-ffunction-sections', '-fdata-sections', '-nostdinc', '-nostdinc++',
             '-DBW_SIMULATION_CODAL_SERVICES=1', '-DBW_PXT_RAFFS_COMPILE=1', '-DSAMD51=1']
    for directory in [ROOT / 'pxt-raffs-compile', out, reviewed,
                      *[core / 'inc' / part for part in ['core', 'types', 'driver-models', 'drivers']]]:
        flags += ['-I', directory.relative_to(ROOT).as_posix()]

    def compile_one(name, source, selected):
        command = ['arm-none-eabi-g++', *selected, '-MD', '-MF', str((out / (name + '.d')).relative_to(ROOT)),
                   '-c', str(source.relative_to(ROOT)), '-o', str((out / (name + '.o')).relative_to(ROOT))]
        result = subprocess.run(command, cwd=ROOT, capture_output=True)
        (out / (name + '.stderr')).write_bytes(result.stderr)
        return result

    negative = compile_one('missing-selector', out / 'RAFFS-storage.cpp',
                           [f for f in flags if f != '-DBW_PXT_RAFFS_COMPILE=1'])
    sources.need(negative.returncode != 0 and
                 b'Select the SAMD51 PXT RAFFS compile-only profile explicitly' in negative.stderr,
                 'missing profile must refuse specifically')
    negative = compile_one('unadapted-raffs', out / 'RAFFS-original.cpp', flags)
    sources.need(negative.returncode != 0 and b'pxtbase.h: No such file or directory' in negative.stderr,
                 'unadapted shared header must not resolve to a stub')
    result = compile_one('RAFFS-storage', out / 'RAFFS-storage.cpp', flags)
    if result.returncode:
        print(result.stderr.decode(errors='replace'))
        raise ValueError('actual include-adapted RAFFS compilation failed')
    source_pin = consumer.git(ROOT, 'rev-parse', 'HEAD').decode().strip()
    rows = []
    for item in consumer.dependencies((out / 'RAFFS-storage.d').read_text()):
        path = (ROOT / item).resolve()
        component, name = origin(path, core, out)
        raw = path.read_bytes()
        if component == 'adapted-raffs':
            expected, pin = adapted, sources.PIN
        elif component == 'extracted-declaration':
            expected, pin = declaration, sources.PIN
        elif component == 'pxt-common-packages':
            expected, pin = payloads[name], sources.PIN
        else:
            checkout, pin = (core, CORE) if component == 'codal-core' else (ROOT, source_pin)
            expected = consumer.git(checkout, 'show', pin + ':' + name)
        sources.need(raw == expected, 'selected input bytes changed')
        rows.append(dict(component=component, pin=pin, path=name, sha256=digest(raw)))
        target = out / 'selected-sources' / component / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)
    symbols = subprocess.check_output(['arm-none-eabi-nm', '-C', '-u', str(out / 'RAFFS-storage.o')]).decode()
    for symbol in ['pxt::hash_fnv1(', 'codal::fiber_wait_for_event(', 'codal::allocateNotifyEvent(',
                   'codal::system_timer_current_time(', 'strlen', 'memcpy', 'memset', 'memcmp']:
        sources.need(symbol in symbols, 'required unresolved real dependency missing: ' + symbol)
    defined = subprocess.check_output(['arm-none-eabi-nm', '-C', '--defined-only', str(out / 'RAFFS-storage.o')]).decode()
    for method in ['FS(', 'write(', 'read(', 'remove(', 'format()', 'tryGC(', 'dirRead()', 'readFlashBytes(']:
        sources.need('pxt::raffs::FS::' + method in defined, 'real RAFFS method missing: ' + method)
    (out / 'undefined-symbols.txt').write_text(symbols)
    (out / 'defined-symbols.txt').write_text(defined)
    notices = out / 'notices'
    notices.mkdir()
    (notices / 'LICENSE-pxt-common-packages').write_bytes(notice)
    for name, source in [('LICENSE-codal-core', core / 'LICENSE'), ('LICENSE-codal-samd', ROOT / 'LICENSE'),
                          ('COPYING3', reviewed / 'COPYING3'), ('COPYING.RUNTIME', reviewed / 'COPYING.RUNTIME')]:
        shutil.copyfile(source, notices / name)
    report = dict(schema=1, status='INCLUDE_ADAPTED_REAL_RAFFS_COMPILE_ONLY',
        runtime_source=source_pin, core_source=CORE, framework_source=sources.PIN,
        original_sha256=digest(original), adapted_sha256=digest(adapted),
        original_pxtbase_sha256=digest(base), extracted_declaration_sha256=digest(declaration),
        replaced_include_lines=2, compiler_flags=flags, source_inputs=rows,
        object_sha256=digest((out / 'RAFFS-storage.o').read_bytes()),
        compiler=subprocess.check_output(['arm-none-eabi-g++', '--version']).decode(),
        missing_selector_refused=True, unadapted_without_pxtbase_stub_refused=True,
        full_pxt_headers_compiled=False, settings_compiled=False, hash_implemented=False,
        linked=False, executed=False, storage_model_qualified=False, image_admitted=False,
        original_board_configuration_qualified=False, licence_clearance=False)
    (out / 'compile-report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ['source_inputs', 'compiler_flags', 'compiler']}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['core', 'framework', 'out']:
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    run(args.core.resolve(), args.framework.resolve(), args.out.resolve())
