#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Hosted settings compilation with real complete PXT base definitions, include-adapted."""
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
import generated
sys.path.insert(0, str(ROOT / 'consumer-build'))
import build as consumer

CORE = consumer.PIN
SETTINGS = 'pxtapp/settings/settings.cpp'
BASE = 'pxtapp/pxtbase.h'
PUBLIC = {'Flash.h':'pxtapp/settings/Flash.h', 'RAFFS.h':'pxtapp/settings/RAFFS.h',
          'configkeys.h':'pxtapp/configkeys.h', 'pxtcore.h':'pxtapp/pxtcore.h'}
OWN = {'pxt-settings-compile/platform_includes.h', 'pxt-settings-compile/pxt-storage-freestanding.h',
       'pxt-settings-compile/settings-abi.cpp', 'runtime-services/platform_includes.h',
       'runtime-services/reviewed-toolchain/stdbool.h'}
SETTINGS_EDITS = [(b'#include "pxt.h"\n', b'#include "pxtbase-storage.h"\n'),
                  (b'#include "GhostFAT.h"\n', b'#include "CodalDevice.h"\n')]
BASE_EDITS = [(b'#include <string.h>\n', 1), (b'#include <stdint.h>\n', 2),
              (b'#include <stddef.h> // offsetof\n', 1), (b'#include <math.h>\n', 1),
              (b'#include <new>\n', 1), (b'#include "platform.h"\n', 1)]


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def adapt(settings, base):
    for before, after in SETTINGS_EDITS:
        sources.need(settings.count(before) == 1, 'unique settings include required')
        settings = settings.replace(before, after, 1)
    # Change includes only. Every type, field, inline method, GC helper, alias,
    # namespace and macro in the complete real base header remains byte-identical.
    for before, count in BASE_EDITS:
        sources.need(base.count(before) == count, 'exact base include census required')
        base = base.replace(before, b'#include "pxt-storage-freestanding.h"\n')
    return settings, base


def origin(path, core, out):
    path = path.resolve()
    if path == out / 'settings-storage.cpp':
        return 'adapted-settings', SETTINGS
    if path == out / 'pxtbase-storage.h':
        return 'adapted-pxtbase', BASE
    if path == out / 'pxtconfig.h':
        return 'reconstructed-config', 'pxtapp/pxtconfig.h'
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


def run(core, framework, generator, out):
    consumer.hosted()
    for path in [core, framework, generator, out]:
        sources.need(path.is_relative_to(ROOT) and path != ROOT and ' ' not in str(path),
                     'owned in-checkout paths required')
    sources.need(not out.exists() and not out.is_symlink(), 'new output required')
    sources.need(consumer.git(core,'rev-parse','HEAD').decode().strip() == CORE and
                 not consumer.git(core,'status','--porcelain','--untracked-files=no'), 'exact clean core required')
    sources.need(not consumer.git(ROOT,'status','--porcelain','--untracked-files=no'), 'dirty runtime')
    payloads, notice = sources.verify(sources.load(ROOT / 'profiles/pxt-framework-sources.json'), framework)
    config, generator_notice = generated.verify(generator)
    config = config['pxtapp/pxtconfig.h']
    original, original_base = payloads[SETTINGS], payloads[BASE]
    adapted, adapted_base = adapt(original, original_base)
    reviewed = ROOT / 'runtime-services/reviewed-toolchain'
    sources.need(digest((reviewed/'stdbool.h').read_bytes()) == consumer.BOOL_SHA, 'reviewed compiler header')
    out.mkdir()
    files = {'settings-original.cpp':original, 'settings-storage.cpp':adapted,
             'pxtbase-original.h':original_base, 'pxtbase-storage.h':adapted_base, 'pxtconfig.h':config,
             **{name:payloads[path] for name,path in PUBLIC.items()}}
    for name, raw in files.items():
        (out/name).write_bytes(raw)
    flags = ['-mcpu=cortex-m4','-mthumb','-std=c++11','-O0','-ffreestanding',
             '-fno-exceptions','-fno-rtti','-fno-unwind-tables','-fno-builtin',
             '-ffunction-sections','-fdata-sections','-nostdinc','-nostdinc++',
             '-DBW_SIMULATION_CODAL_SERVICES=1','-DBW_PXT_SETTINGS_COMPILE=1','-DSAMD51=1']
    for directory in [ROOT/'pxt-settings-compile',out,reviewed,
                      *[core/'inc'/part for part in ['core','types','driver-models','drivers']]]:
        flags += ['-I',str(directory.relative_to(ROOT))]

    def compile_one(name, source, selected):
        command = ['arm-none-eabi-g++',*selected,'-MD','-MF',str((out/(name+'.d')).relative_to(ROOT)),
                   '-c',str(source.relative_to(ROOT)),'-o',str((out/(name+'.o')).relative_to(ROOT))]
        result = subprocess.run(command,cwd=ROOT,capture_output=True)
        (out/(name+'.stderr')).write_bytes(result.stderr)
        return result

    negative = compile_one('missing-selector',out/'settings-storage.cpp',
                           [f for f in flags if f != '-DBW_PXT_SETTINGS_COMPILE=1'])
    sources.need(negative.returncode != 0 and
                 b'Select the SAMD51 PXT settings compile-only profile explicitly' in negative.stderr,
                 'missing selector must refuse specifically')
    negative = compile_one('unadapted-settings',out/'settings-original.cpp',flags)
    sources.need(negative.returncode != 0 and b'pxt.h: No such file or directory' in negative.stderr,
                 'original must not resolve through a fake PXT header')
    units = [('settings-storage',out/'settings-storage.cpp'),
             ('settings-abi',ROOT/'pxt-settings-compile/settings-abi.cpp')]
    for name, path in units:
        result = compile_one(name,path,flags)
        if result.returncode:
            print(result.stderr.decode(errors='replace'))
            raise ValueError('actual settings/base ABI compilation failed: '+name)
    source_pin = consumer.git(ROOT,'rev-parse','HEAD').decode().strip()
    rows, seen = [], set()
    for name,_ in units:
        for item in consumer.dependencies((out/(name+'.d')).read_text()):
            path = (ROOT/item).resolve()
            component, public = origin(path,core,out)
            if (component,public) in seen:
                continue
            seen.add((component,public))
            raw = path.read_bytes()
            if component == 'adapted-settings':
                expected,pin = adapted,sources.PIN
            elif component == 'adapted-pxtbase':
                expected,pin = adapted_base,sources.PIN
            elif component == 'reconstructed-config':
                expected,pin = config,generated.PIN
            elif component == 'pxt-common-packages':
                expected,pin = payloads[public],sources.PIN
            else:
                checkout,pin = (core,CORE) if component == 'codal-core' else (ROOT,source_pin)
                expected = consumer.git(checkout,'show',pin+':'+public)
            sources.need(raw == expected,'selected dependency bytes changed')
            rows.append(dict(component=component,pin=pin,path=public,sha256=digest(raw)))
            target = out/'selected-sources'/component/public
            target.parent.mkdir(parents=True,exist_ok=True)
            target.write_bytes(raw)
    required = {('adapted-settings',SETTINGS),('adapted-pxtbase',BASE),
                ('reconstructed-config','pxtapp/pxtconfig.h'),
                *[('pxt-common-packages',path) for path in PUBLIC.values()]}
    sources.need(required <= seen,'real settings/base/config/interface census')
    symbols = subprocess.check_output(['arm-none-eabi-nm','-C','-u',str(out/'settings-storage.o')]).decode()
    for symbol in ['pxt::mkBuffer(', 'pxt::mkString(', 'pxt::registerGCPtr(', 'pxt::unregisterGCPtr(',
                   'pxt::Segment::set(', 'Array_::mk()', 'pxt::getConfig(',
                   'pxt::raffs::FS::write(', 'pxt::raffs::FS::read(', 'memcmp']:
        sources.need(symbol in symbols,'real unresolved settings dependency missing: '+symbol)
    defined = subprocess.check_output(['arm-none-eabi-nm','-C','--defined-only',str(out/'settings-storage.o')]).decode()
    for method in ['_set(', '_get(', '_remove(', '_exists(', '_list(', '_userClean()',
                   'largeStoreStart()', 'largeStoreSize()', 'largeStoreFlash()']:
        sources.need('settings::'+method in defined,'real settings method missing: '+method)
    (out/'undefined-symbols.txt').write_text(symbols)
    (out/'defined-symbols.txt').write_text(defined)
    notices = out/'notices'
    notices.mkdir()
    (notices/'LICENSE-pxt-common-packages').write_bytes(notice)
    (notices/'LICENSE-pxt-generator').write_bytes(generator_notice)
    for name,path in [('LICENSE-codal-core',core/'LICENSE'),('LICENSE-codal-samd',ROOT/'LICENSE'),
                      ('COPYING3',reviewed/'COPYING3'),('COPYING.RUNTIME',reviewed/'COPYING.RUNTIME')]:
        shutil.copyfile(path,notices/name)
    report = dict(schema=1,status='INCLUDE_ADAPTED_REAL_SETTINGS_AND_PXTBASE_COMPILE_ONLY',
        runtime_source=source_pin,core_source=CORE,framework_source=sources.PIN,generator_source=generated.PIN,
        adaptations=dict(settings_include_lines=2,pxtbase_include_lines=7,
            original_settings_sha256=digest(original),adapted_settings_sha256=digest(adapted),
            original_pxtbase_sha256=digest(original_base),adapted_pxtbase_sha256=digest(adapted_base)),
        config_sha256=digest(config),compiler_flags=flags,source_inputs=rows,
        objects={name:digest((out/(name+'.o')).read_bytes()) for name,_ in units},
        compiler=subprocess.check_output(['arm-none-eabi-g++','--version']).decode(),
        missing_selector_refused=True,unadapted_without_pxt_stub_refused=True,
        original_pxt_class_bodies_preserved=True,selected_base_abi_compile_checks=True,
        full_samd_platform_compiled=False,gc_implementation_linked=False,linked=False,executed=False,
        original_board_configuration_qualified=False,storage_model_qualified=False,
        image_admitted=False,licence_clearance=False)
    (out/'compile-report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ['source_inputs','compiler_flags','compiler']}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['core','framework','generator','out']:
        parser.add_argument(name,type=Path)
    args = parser.parse_args()
    run(args.core.resolve(),args.framework.resolve(),args.generator.resolve(),args.out.resolve())
