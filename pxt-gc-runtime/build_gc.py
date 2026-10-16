#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Hosted real GC/services link, not native Arcade or guest qualification."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools/pxt-storage'))
import sources
import generated
sys.path.insert(0, str(ROOT / 'consumer-build'))
import build as consumer

CORE_SOURCES = ['source/driver-models/Timer.cpp', 'source/core/CodalComponent.cpp',
                'source/core/CodalHeapAllocator.cpp', 'source/core/CodalFiber.cpp',
                'source/core/CodalListener.cpp', 'source/drivers/MessageBus.cpp',
                'source/core/MemberFunctionCallback.cpp', 'source/types/Event.cpp']
OWN_SOURCES = ['runtime-services/clock.cpp', 'pxt-native-allocation/native-aligned-heap.cpp',
               'pxt-gc-runtime/libc.cpp', 'pxt-gc-runtime/probe.cpp']
BASE_EDITS = [(b'#include <string.h>\n', 1), (b'#include <stdint.h>\n', 2),
              (b'#include <stddef.h> // offsetof\n', 1), (b'#include <math.h>\n', 1),
              (b'#include <new>\n', 1), (b'#include "platform.h"\n', 1)]
REALLOC_COPY = b'memcpy(mem, ptr, min(blockSize * sizeof(void *), size));'
REALLOC_PAYLOAD_COPY = b'memcpy(mem, ptr, min((blockSize - 1) * sizeof(void *), size));'
PUBLIC_MALLOC = b'void *malloc(size_t sz) {\n    if (PXT_IN_ISR() || inGC)\n        return xmalloc(sz);\n    else\n        return app_alloc(sz);\n}'
PUBLIC_MALLOC_GUARDED = b'void *malloc(size_t sz) {\n    if (PXT_IN_ISR() || inGC)\n        return xmalloc(sz);\n    else {\n        // Refuse before size_t-to-int conversion and array-header arithmetic.\n        static_assert(GC_MAX_ALLOC_SIZE >= sizeof(void *) &&\n                      GC_MAX_ALLOC_SIZE % sizeof(void *) == 0, "word-aligned GC cap");\n        if (sz > GC_MAX_ALLOC_SIZE - sizeof(void *))\n            soft_panic(PANIC_GC_TOO_BIG_ALLOCATION);\n        return app_alloc(sz);\n    }\n}'
PUBLIC_SIZE_CASES = ['public-size-cap', 'public-size-conversion',
                     'public-size-cap-exact', 'public-size-signed-aligned',
                     'public-size-signed-max', 'public-size-signed-over',
                     'public-size-wrap-aligned', 'public-size-recovery']


def selected_probe(case):
    sources.need(case in ['baseline', 'allocation-routing', 'allocation-churn', 'native-real-oom', 'public-gc-real-oom', *PUBLIC_SIZE_CASES], 'unknown GC caller case')
    if case == 'baseline':
        return list(OWN_SOURCES), 'pxt-gc-runtime/startup.S'
    if case == 'allocation-routing':
        return [*OWN_SOURCES[:-1], 'pxt-gc-runtime/probe-routing.cpp'], 'pxt-gc-runtime/routing-startup.S'
    if case == 'native-real-oom':
        return [*OWN_SOURCES[:-1], 'pxt-gc-runtime/probe-native-oom.cpp'], 'pxt-gc-runtime/startup.S'
    if case == 'public-gc-real-oom':
        return [*OWN_SOURCES[:-1], 'pxt-gc-runtime/probe-public-oom.cpp'], 'pxt-gc-runtime/startup.S'
    if case in PUBLIC_SIZE_CASES:
        return [*OWN_SOURCES[:-1], 'pxt-gc-runtime/probe-' + case + '.cpp'], 'pxt-gc-runtime/startup.S'
    return [*OWN_SOURCES[:-1], 'pxt-gc-runtime/probe-churn.cpp'], 'pxt-gc-runtime/startup.S'


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def once(raw, before, after):
    sources.need(raw.count(before) == 1, 'unique exact adaptation required')
    return raw.replace(before, after, 1)


def between(raw, start, end):
    sources.need(raw.count(start) == raw.count(end) == 1, 'unique extraction boundaries required')
    a, b = raw.index(start), raw.index(end)
    sources.need(a < b, 'reversed extraction boundaries')
    return raw[a:b]


def native_calls(raw, allocations, frees, include):
    sources.need(len(re.findall(rb'\bmalloc\(', raw)) == allocations and
                 len(re.findall(rb'\bfree\(', raw)) == frees,
                 'exact native allocation call census required')
    raw = once(raw, include, include + b'#include "native-aligned-heap.h"\n')
    raw = re.sub(rb'\bmalloc\(', b'bw_native_alloc(', raw)
    return re.sub(rb'\bfree\(', b'bw_native_free(', raw)


def check_memset_codegen(code):
    sources.need(b'<memset>:' in code and
                 re.search(rb'\bbl(?:x|\.[nw])?\s', code) is None and
                 re.search(rb'\bb(?:\.[nw])?\s+[0-9a-f]+\s+<memset>', code) is None and
                 re.search(rb'R_ARM_\S+\s+memset\b', code) is None and
                 b'R_ARM_THM_CALL' not in code and b'R_ARM_CALL' not in code,
                 'freestanding memset must not contain a generated call')


def core_adapt(core):
    originals = {name: (core / path).read_bytes() for name, path in
                 [('fiber-native.cpp', 'source/core/CodalFiber.cpp'),
                  ('timer-native.cpp', 'source/driver-models/Timer.cpp'),
                  ('heap-no-public-aliases.cpp', 'source/core/CodalHeapAllocator.cpp')]}
    heap = originals['heap-no-public-aliases.cpp']
    for line in [b'void *malloc(size_t sz) __attribute__ ((weak, alias ("device_malloc")));\n',
                 b'void free(void *mem) __attribute__ ((weak, alias ("device_free")));\n',
                 b'void* realloc (void* ptr, size_t size) __attribute__ ((weak, alias ("device_realloc")));\n']:
        heap = once(heap, line, b'')
    return originals, {
        'fiber-native.cpp': native_calls(originals['fiber-native.cpp'], 1, 4,
                                        b'#include "CodalConfig.h"\n'),
        'timer-native.cpp': native_calls(originals['timer-native.cpp'], 1, 0,
                                        b'#include "Timer.h"\n'),
        'heap-no-public-aliases.cpp': heap}


def adapt(payloads, target):
    base = payloads['pxtapp/pxtbase.h']
    for before, count in BASE_EDITS:
        sources.need(base.count(before) == count, 'exact base include census required')
        base = base.replace(before, b'#include "pxt-gc-freestanding.h"\n')
    gc = once(payloads['pxtapp/base/gc.cpp'], b'#include "pxtbase.h"\n',
              b'#include "pxtbase-storage.h"\n')
    # VAR_BLOCK_WORDS includes the array header; ptr points after that word.
    # Preserve the original source and require this exact sole copy site.
    gc = once(gc, REALLOC_COPY, REALLOC_PAYLOAD_COPY)
    gc = once(gc, PUBLIC_MALLOC, PUBLIC_MALLOC_GUARDED)
    objects = once(payloads['pxtapp/base/pxt.cpp'], b'#include "pxtbase.h"\n',
                   b'#include "pxtbase-storage.h"\n')
    objects = once(objects, b'using namespace std;\n',
                   b'// No standard namespace symbols are used here.\n')
    codal = payloads['pxtapp/core---samd/codal.cpp']
    listing = between(codal, b'#ifdef DEVICE_GET_FIBER_LIST_AVAILABLE\n',
                      b'\nnamespace pxt {\n')
    roots = between(codal, b'ThreadContext *getThreadContext() {\n',
                    b'LowLevelTimer *getJACDACTimer() {\n')
    roots = (b'#include "pxtbase-storage.h"\n#include "CodalFiber.h"\n'
             b'using namespace codal;\n' + listing + b'\nnamespace pxt {\n' + roots + b'}\n')
    config = between(payloads['pxtapp/base/core.cpp'],
                     b'int *getBootloaderConfigData() __attribute__((weak));\n',
                     b'\n} // namespace pxt\n\nnamespace pxtrt {\n')
    config = b'#include "pxtbase-storage.h"\nnamespace pxt {\n' + config + b'\n}\n'
    # Native C++ objects/TCBs must retain an eight-byte-aligned allocation
    # domain, while PXT placement-new objects retain actual PXT GC ownership.
    target = native_calls(target, 2, 4, b'#include "Runtime.h"\n')
    # Optimize only the existing byte-clearing function in this new profile.
    # The body, all other target functions, and service-only builds stay intact.
    target = once(target, b'extern "C" void *memset(',
                  b'extern "C" __attribute__((optimize("O2", "no-tree-loop-distribute-patterns"))) void *memset(')
    return {'pxtbase-storage.h': base, 'gc-storage.cpp': gc,
            'pxt-object-storage.cpp': objects, 'pxt-fiber-roots.cpp': roots,
            'pxt-config.cpp': config, 'target-native.cpp': target}


def run(core, framework, generator, out, case='baseline'):
    consumer.hosted()
    own_sources, startup_source = selected_probe(case)
    for path in [core, framework, generator, out]:
        sources.need(path.is_relative_to(ROOT) and path != ROOT and ' ' not in str(path),
                     'owned in-checkout paths required')
    sources.need(not out.exists() and not out.is_symlink(), 'new output required')
    sources.need(consumer.git(core, 'rev-parse', 'HEAD').decode().strip() == consumer.PIN and
                 not consumer.git(core, 'status', '--porcelain', '--untracked-files=no'), 'clean exact core')
    sources.need(not consumer.git(ROOT, 'status', '--porcelain', '--untracked-files=no'), 'dirty runtime')
    payloads, framework_notice = sources.verify(
        sources.load(ROOT / 'profiles/pxt-framework-sources.json'), framework)
    generated_files, generator_notice = generated.verify(generator)
    staged = adapt(payloads, (ROOT / 'runtime-services/target.cpp').read_bytes())
    staged['pxtconfig.h'] = generated_files['pxtapp/pxtconfig.h']
    for name in ['configkeys.h', 'pxtcore.h']:
        staged[name] = payloads['pxtapp/' + name]
    out.mkdir()
    # The always-upload failure packet also contains selected public sources.
    # Retain their notices before staging any such input, not only after link.
    notices = out / 'notices'
    notices.mkdir()
    (notices / 'LICENSE-pxt-common-packages').write_bytes(framework_notice)
    (notices / 'LICENSE-pxt-generator').write_bytes(generator_notice)
    shutil.copyfile(core / 'LICENSE', notices / 'LICENSE-codal-core')
    shutil.copyfile(ROOT / 'LICENSE', notices / 'LICENSE-codal-samd')
    for name in ['COPYING.RUNTIME', 'COPYING3']:
        shutil.copyfile(ROOT / 'runtime-services/reviewed-toolchain' / name, notices / name)
    for name, raw in staged.items():
        (out / name).write_bytes(raw)
    originals = out / 'original-public'
    originals.mkdir()
    for path in ['pxtapp/pxtbase.h', 'pxtapp/base/gc.cpp', 'pxtapp/base/pxt.cpp',
                 'pxtapp/core---samd/codal.cpp', 'pxtapp/base/core.cpp']:
        (originals / path.replace('/', '-')).write_bytes(payloads[path])
    commands = []

    def command(argv, label):
        argv = [str(a) for a in argv]
        result = subprocess.run(argv, cwd=ROOT, capture_output=True, timeout=120)
        (out / (label + '.stdout')).write_bytes(result.stdout)
        (out / (label + '.stderr')).write_bytes(result.stderr)
        commands.append({'label': label, 'argv': argv, 'returncode': result.returncode})
        (out / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        sources.need(result.returncode == 0, 'actual command failed: ' + label)
        return result.stdout

    patches = [ROOT / 'runtime-services' / name for name in
               ['fiber-guards.patch', 'timer-width.patch', 'heap-domains.patch']]
    command(['git', '-C', core, 'apply', '--check', *patches], 'patch-check')
    command(['git', '-C', core, 'apply', *patches], 'patch-apply')
    (out / 'actual-core-patch.diff').write_bytes(consumer.git(core, 'diff', '--binary'))
    core_originals, core_staged = core_adapt(core)
    for name, raw in core_originals.items():
        (originals / ('patched-core-' + name)).write_bytes(raw)
    staged.update(core_staged)
    for name, raw in core_staged.items():
        (out / name).write_bytes(raw)
    reviewed = ROOT / 'runtime-services/reviewed-toolchain'
    sources.need(digest((reviewed / 'stdbool.h').read_bytes()) == consumer.BOOL_SHA, 'reviewed bool header')
    flags = ['-mcpu=cortex-m4', '-mthumb', '-std=c++11', '-O0', '-ffreestanding',
             '-fno-exceptions', '-fno-rtti', '-fno-unwind-tables', '-fno-asynchronous-unwind-tables',
             '-fno-builtin', '-ffunction-sections', '-fdata-sections', '-nostdinc', '-nostdinc++',
             '-DBW_SIMULATION_CODAL_SERVICES=1', '-DBW_PXT_GC_LINK=1',
             '-DBW_PXT_NATIVE_ALLOCATION=1', '-DSAMD51=1']
    directories = [ROOT / 'pxt-gc-runtime', out, ROOT / 'pxt-native-allocation',
                   ROOT / 'runtime-services', ROOT / 'simulation/include',
                   ROOT / 'paired-irq/include', reviewed,
                   *[core / 'inc' / p for p in ['core', 'types', 'driver-models', 'drivers']]]
    for directory in directories:
        flags += ['-I', directory.relative_to(ROOT).as_posix()]
    objects = []
    replaced = {'CodalFiber.cpp', 'Timer.cpp', 'CodalHeapAllocator.cpp'}
    inputs = [core / p for p in CORE_SOURCES if Path(p).name not in replaced]
    inputs += [ROOT / p for p in own_sources]
    inputs += [out / n for n in staged if n.endswith('.cpp')]
    dependencies = {}
    for index, path in enumerate(inputs):
        name = str(index) + '-' + path.stem
        obj = out / (name + '.o')
        command(['arm-none-eabi-g++', *flags, '-MD', '-MF', out / (name + '.d'),
                 '-c', path.relative_to(ROOT), '-o', obj.relative_to(ROOT)], name)
        objects.append(obj.relative_to(ROOT))
        for dep in consumer.dependencies((out / (name + '.d')).read_text()):
            selected = (ROOT / dep).resolve()
            if selected.is_relative_to(core):
                origin = 'codal-core-patched'
            elif selected.is_relative_to(out):
                sources.need(selected.name in staged, 'unknown generated dependency')
                origin = 'explicit-adaptation'
            else:
                sources.need(selected.is_relative_to(ROOT) and selected.suffix in ['.h', '.cpp'] and
                             selected.relative_to(ROOT).parts[0] in
                             ['runtime-services', 'pxt-gc-runtime', 'pxt-native-allocation',
                              'simulation', 'paired-irq'], 'external/vendor header refused')
                origin = 'codal-samd'
            dependencies[dep] = {'origin': origin, 'sha256': digest(selected.read_bytes())}
        symbols = command(['arm-none-eabi-nm', '-g', '--defined-only', obj.relative_to(ROOT)], name + '-symbols')
        if path.name == 'target-native.cpp':
            code = command(['arm-none-eabi-objdump', '-dr', '--disassemble=memset',
                            obj.relative_to(ROOT)], 'memset-codegen')
            check_memset_codegen(code)
        definitions = {line.split()[-1] for line in symbols.splitlines() if line.split()}
        api = definitions & {b'malloc', b'free', b'realloc'}
        sources.need(api == ({b'malloc', b'free', b'realloc'} if path.name == 'gc-storage.cpp' else set()),
                     'PXT GC must be sole public allocator owner')
    for name, source in [('startup', ROOT / startup_source),
                         ('context', ROOT / 'asm/CortexContextSwitch.s')]:
        command(['arm-none-eabi-gcc', '-mcpu=cortex-m4', '-mthumb', '-c',
                 source.relative_to(ROOT), '-o', (out / (name + '.o')).relative_to(ROOT)], name)
        objects.append((out / (name + '.o')).relative_to(ROOT))
    elf = (out / 'gc-control.elf').relative_to(ROOT)
    command(['arm-none-eabi-g++', '-mcpu=cortex-m4', '-mthumb', '-nostdlib', '-nostartfiles',
             '-Wl,--gc-sections,--no-undefined,-Map=' + str((out / 'gc-control.map').relative_to(ROOT)),
             '-T', 'pxt-gc-runtime/control.ld', *objects, '-lgcc', '-o', elf], 'link')
    sources.need(not command(['arm-none-eabi-nm', '-u', elf], 'undefined').strip(), 'unresolved image')
    sections = command(['arm-none-eabi-objdump', '-h', elf], 'sections')
    sources.need(b'.init_array' in sections, 'actual GC static constructors must be retained')
    command(['arm-none-eabi-nm', '-S', elf], 'symbols')
    command(['arm-none-eabi-objdump', '-d', elf], 'disassembly')
    command(['arm-none-eabi-objcopy', '-O', 'binary', elf,
             (out / 'gc-control.bin').relative_to(ROOT)], 'binary')
    command(['arm-none-eabi-g++', '--version'], 'compiler')
    command(['arm-none-eabi-g++', '-mcpu=cortex-m4', '-mthumb', '-print-libgcc-file-name'], 'libgcc-path')
    record = {'schema': 1, 'status': 'GC_SERVICES_LINK_ONLY',
              'source': consumer.git(ROOT, 'rev-parse', 'HEAD').decode().strip(),
              'core': consumer.PIN, 'framework': sources.PIN,
              'staged': {n: digest(b) for n, b in staged.items()}, 'dependencies': dependencies,
              'elf_sha256': digest((out / 'gc-control.elf').read_bytes()),
              'binary_sha256': digest((out / 'gc-control.bin').read_bytes()),
              'guest_qualified': False, 'native_arcade': False, 'performance_measured': False,
              'board_qualified': False, 'licence_clearance': False}
    if case != 'baseline':
        record.update(status={'allocation-routing': 'GC_ALLOCATION_ROUTING_LINK_ONLY',
                              'allocation-churn': 'GC_ALLOCATION_CHURN_LINK_ONLY',
                              'native-real-oom': 'GC_NATIVE_REAL_OOM_LINK_ONLY',
                              'public-gc-real-oom': 'GC_PUBLIC_REAL_OOM_LINK_ONLY',
                              **{name: 'GC_PUBLIC_SIZE_LINK_ONLY' for name in PUBLIC_SIZE_CASES}}[case], case=case,
                      probe_source=own_sources[-1], startup_source=startup_source)
    (out / 'build-report.json').write_text(json.dumps(record, indent=2, sort_keys=True) + '\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['core', 'framework', 'generator', 'out']:
        parser.add_argument(name, type=lambda value: Path(value).resolve())
    parser.add_argument('--case', choices=['baseline', 'allocation-routing', 'allocation-churn', 'native-real-oom', 'public-gc-real-oom', *PUBLIC_SIZE_CASES], default='baseline')
    args = parser.parse_args()
    run(args.core, args.framework, args.generator, args.out, args.case)
