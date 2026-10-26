#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Compile and link the complete no-ASF SAMD51 board base from its admitted inputs.

Inputs are byte-bound: the reproduced MakeCode native request (Arcade 4.2.1 /
PXT 13.2.1, samd51adafruit), pinned codal-core and board checkouts, the reproduced
forced configuration header, generated Apache-2.0 ATDF device headers and this
repository's own sources. Every compiler dependency of every translation unit is
admitted into exactly one category; ASF4, samd-peripherals and vendor-restricted
device headers are refused. The output base is an image, not a qualified board.
"""
import argparse
import hashlib
import importlib.util
import json
import os
import re
import shutil
import subprocess
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CORE = '2ed19d64ec3359143409a4b0a9758b4aeef3ee7d'
BOARD = '6ecd80ccf1abcc126d06653f178ca03a4d1c6421'
# Admitted native requests: the empty Arcade project, and a program using the
# serial extension (base bindings plus a delta profile for the files it changes).
VARIANTS = {
    'base': dict(request_sha256='19efcdc73769fdfdeb51aa215c528bebad59782cbc538f72f4a194326f1f42b1', files=69, delta=None),
    'serial': dict(request_sha256='46f71f70a05d56d9c86c813c8d949fbc30521e2fa8cf948a3790866fe1bf3213', files=73,
                   delta='profiles/pxt-framework-sources-serial.json'),
    # No MakeCode runtime: the same core, drivers and support layer with the
    # TCC timer leaf's own main (MakeCode cannot select a TCC system timer).
    'tcc-leaf': dict(request_sha256=None, files=0, delta=None, leaf='no-asf-board/tcc-leaf'),
}
ATDF_SHA256 = '9b21d97587c78c217842048aedfe5a3d394b0a71f741da3c40dc6629892c6fbd'
# CMSIS 4.30 core headers (BSD-3-Clause, retained in inc/CMSIS); nothing else there.
CMSIS = ['core_cm4.h', 'core_cmFunc.h', 'core_cmInstr.h', 'core_cmSimd.h', 'cmsis_gcc.h']
# Units present in a selected directory that this board cannot run, with the reason.
INACTIVE = {
    'src/CapTouchButton.cpp': 'SAMD21 PTC touch controller; the admitted SAMD51 ATDF has no PTC module',
    'src/adafruit_ptc.cpp': 'SAMD21 PTC touch controller; the admitted SAMD51 ATDF has no PTC module',
    'board:src/startup_samd51.c': 'names TAL_0/TAL_1 vectors absent from the admitted ATDF; replaced by the '
                                  'generated vector table and no-asf-board/support/startup.c',
}
# sendConfig() allocates its descriptor buffer with new[]; the reviewed adaptation frees it with delete[].
CORE_ADAPTATION = ('source/driver-models/CodalUSB.cpp', 'dc2b31fd5919d553921354a0519dde8a42e264e006be6a00bcce9ad081a89b8d',
                   b'    delete buf;\n', b'    delete[] buf;\n')
# The original request remains byte-bound. The compiled owned copy changes only
# the factory discriminator; the original-index variant is the omission control.
TIMER_ADAPTATION = ('pxtapp/core---samd/platform.cpp',
                    'b813dfdeacd5e528e38a5d7286bb60c31c1b373172b2449bfbe1d047431d78fc',
                    b'        if (idx < 0x20) {', b'        if (tcId < 0x20) {')
# Optional, explicitly authored RGB444 simulation contract. This is an actual
# guest upload, never a model power-on default or a statement about module glass.
COLOUR_SOURCE = ('pxtapp/screen---st7735/screen.cpp', 'dea9ea175d65d885275eb0715d56353674feae88d64d29a5eb2809f899a0d958')
COLOUR_ANCHOR = b'                wrCmd1(0x20, 0x00); // INVOFF\n'


def colour_source(raw, profile):
    need(sha(raw) == COLOUR_SOURCE[1] and raw.count(COLOUR_ANCHOR) == 1,
         'one exact admitted direct RGB444 constructor')
    if profile == 'original':
        return raw
    need(profile in ('rgb444-lut', 'short-lut'), 'declared colour profile')
    count = 128 if profile == 'rgb444-lut' else 127
    # R32/G64/B32: the first sixteen entries expand a 4-bit value to 6 bits;
    # unused entries saturate. RGB565 is explicitly outside this profile.
    extra = ("                // Owned RGB444 simulation profile: explicit RGBSET, not a panel default.\n"
             "                dcPin_->setDigitalValue(0); csPin_->setDigitalValue(0); spi_->write(0x2d);\n"
             "                dcPin_->setDigitalValue(1);\n"
             "                for (int n = 0; n < %d; ++n) {\n"
             "                    int i = n < 32 ? n : (n < 96 ? n - 32 : n - 96);\n"
             "                    if (i > 15) i = 15;\n"
             "                    spi_->write((i << 2) | (i >> 2));\n"
             "                }\n"
             "                csPin_->setDigitalValue(1);\n") % count
    return raw.replace(COLOUR_ANCHOR, COLOUR_ANCHOR + extra.encode())


CPU = ['-mcpu=cortex-m4', '-mthumb', '-mfloat-abi=softfp', '-mfpu=fpv4-sp-d16']
# Board target flags (codal-itsybitsy-m4 target.json) at the historical -O2. The
# DWARF-only flag avoids a gas failure on gcc >= 12 variable-location views.
COMMON = CPU + ['-O2', '-g', '-gno-variable-location-views', '-ffunction-sections', '-fdata-sections',
                '-fwrapv', '-fno-exceptions', '-fno-unwind-tables']
CXX = ['-std=c++11', '-fno-rtti', '-fno-threadsafe-statics', '-Wno-array-bounds']
CC = ['-std=c99', '-Warray-bounds']
JUMP_TABLE_MAGIC = '0108010842424242010801083ED8E98D'


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def need(ok, reason):
    if not ok:
        raise ValueError(reason)


def git(folder, *args):
    return subprocess.check_output(['git', '-C', str(folder), *args], text=True).strip()


class Build:
    def __init__(self, args):
        self.args = args
        self.out = args.out.resolve()
        self.commands = []
        self.admitted = {}
        self.digests = {}

    def command(self, argv, label, cwd=None):
        argv = [str(x) for x in argv]
        p = subprocess.run(argv, cwd=cwd or ROOT, capture_output=True, timeout=600)
        log = self.out / 'logs'
        (log / f'{label}.stdout').write_bytes(p.stdout)
        (log / f'{label}.stderr').write_bytes(p.stderr)
        self.commands.append(dict(label=label, argv=argv, returncode=p.returncode))
        need(p.returncode == 0, f'command failed: {label}')
        return p.stdout

    def inputs(self):
        a = self.args
        need(git(a.core, 'rev-parse', 'HEAD') == CORE and not git(a.core, 'status', '--porcelain', '--untracked-files=no'),
             'clean exact codal-core')
        need(git(a.board, 'rev-parse', 'HEAD') == BOARD, 'exact board source')
        variant = VARIANTS[a.variant]
        if variant.get('leaf'):
            need(a.request is None, 'a leaf variant takes no native request')
            return dict(replaceFiles={})
        raw = a.request.read_bytes()
        need(sha(raw) == variant['request_sha256'], 'request bytes are the admitted native request')
        request = json.loads(raw)
        need(request['config'] == 'codal2samd51' and request['tag'] == 'v0.8.0', 'request service and generator tag')
        profile = json.loads((ROOT / 'profiles/pxt-framework-sources.json').read_bytes())
        framework = {s['request']: s for s in profile['sources']}
        unresolved = {u['request']: u for u in profile['unresolved']}
        if variant['delta']:
            delta = json.loads((ROOT / variant['delta']).read_bytes())
            need(delta['request_sha256'] == variant['request_sha256'] and delta['package'] == profile['package'],
                 'delta profile for this request and package')
            for row in delta['sources']:
                unresolved.pop(row['request'], None)
                framework[row['request']] = row
            for row in delta['unresolved']:
                framework.pop(row['request'], None)
                unresolved[row['request']] = row
        files = request['replaceFiles']
        need(len(files) == len(framework) + len(unresolved) == variant['files'], 'complete request file set')
        tree = self.out / 'request'
        for name, text in files.items():
            data = text.encode()
            bound = framework.get(name) or unresolved.get(name)
            need(bound and sha(data) == bound['sha256'] and len(data) == bound['bytes'], 'request file identity: ' + name)
            dest = tree / name.lstrip('/')
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(data)
        report = json.loads((a.device / 'generator.json').read_text())
        need(report['atdf_sha256'] == ATDF_SHA256, 'device headers generated from the admitted ATDF')
        need('PTC' not in report['modules'] and 'TAL' not in report['interrupts'], 'inactive-unit premises hold')
        startup = (a.board / 'src/startup_samd51.c').read_bytes()
        need(b'pfnTAL_0_Handler' in startup, 'board startup still names the absent TAL vectors')
        return request

    def configuration(self):
        spec = importlib.util.spec_from_file_location('source_configuration', ROOT / 'tools/generated-board-profile/verify.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module.verify(self.args.generator, self.args.board, self.out)

    def adapt_core(self):
        path, expected, before, after = CORE_ADAPTATION
        source = self.args.core / path
        raw = source.read_bytes()
        need(sha(raw) == expected and raw.count(before) == 1, 'one exact pinned deallocator adaptation')
        source.write_bytes(raw.replace(before, after))
        need(git(self.args.core, 'diff', '--name-only').splitlines() == [path], 'only the reviewed core adaptation')
        return dict(path=path, original_sha256=expected, adapted_sha256=sha(source.read_bytes()))

    def adapt_request(self):
        if VARIANTS[self.args.variant].get('leaf'):
            return None
        path, expected, before, after = TIMER_ADAPTATION
        source = self.out / 'request' / path
        raw = source.read_bytes()
        need(sha(raw) == expected and raw.count(before) == 1,
             'one exact original timer factory discriminator')
        adapted = raw if self.args.timer_selection == 'original-index' else raw.replace(before, after)
        source.write_bytes(adapted)
        return dict(path=path, original_sha256=expected, adapted_sha256=sha(adapted),
                    selection=self.args.timer_selection,
                    substitution_count=0 if adapted == raw else 1)

    def adapt_colour(self):
        if VARIANTS[self.args.variant].get('leaf'):
            need(self.args.colour_profile == 'original', 'no colour profile in the timer leaf')
            return None
        path, expected = COLOUR_SOURCE
        source = self.out / 'request' / path
        raw = source.read_bytes()
        adapted = colour_source(raw, self.args.colour_profile)
        source.write_bytes(adapted)
        return dict(path=path, original_sha256=expected, adapted_sha256=sha(adapted),
                    profile=self.args.colour_profile,
                    payload_bytes={'original': 0, 'rgb444-lut': 128, 'short-lut': 127}[self.args.colour_profile])

    def units(self):
        a = self.args
        units = sorted((self.out / 'request').rglob('*.cpp'))
        leaf = VARIANTS[a.variant].get('leaf')
        if leaf:
            units += sorted((ROOT / leaf).glob('*.cpp'))
        units += sorted((a.core / 'source').rglob('*.cpp'))
        own = sorted((ROOT / 'src').glob('*.c*'))
        units += [p for p in own if p.relative_to(ROOT).as_posix() not in INACTIVE]
        board = sorted((a.board / 'src').glob('*.c*')) + sorted((a.board / 'model').glob('*.cpp'))
        units += [p for p in board if 'board:' + p.relative_to(a.board).as_posix() not in INACTIVE]
        support = ROOT / 'no-asf-board/support'
        units += sorted(support.glob('*.c')) + sorted(support.glob('*.cpp')) + sorted((support / 'samd').glob('*.c'))
        units += sorted(a.device.glob('*.c'))
        return units

    def includes(self):
        a = self.args
        dirs = [ROOT / 'no-asf-board/support', a.device, self.out / 'cmsis', ROOT / 'inc', self.out / 'libraries']
        if (self.out / 'request/pxtapp').is_dir():
            dirs += sorted({p for p in (self.out / 'request/pxtapp').rglob('*') if p.is_dir()} | {self.out / 'request/pxtapp'})
        dirs += sorted({p.parent for p in (a.core / 'inc').rglob('*.h')})
        dirs += [a.board / 'inc', a.board / 'model']
        (self.out / 'libraries').mkdir(exist_ok=True)
        os.symlink(a.core.resolve(), self.out / 'libraries/codal-core')
        (self.out / 'cmsis').mkdir(exist_ok=True)
        for name in CMSIS:
            shutil.copyfile(ROOT / 'inc/CMSIS' / name, self.out / 'cmsis' / name)
        out = []
        for d in dirs:
            out += ['-I', str(d)]
        return out

    def toolchain_roots(self):
        p = subprocess.run(['arm-none-eabi-gcc', *CPU, '-xc', '-E', '-v', os.devnull], capture_output=True, text=True)
        text = p.stderr.split('#include <...> search starts here:')[1].split('End of search list.')[0]
        return [Path(line.strip()).resolve() for line in text.strip().splitlines()]

    def admit(self, path, toolchain):
        path = Path(path)
        path = path if path.is_absolute() else ROOT / path
        real = path.resolve()
        if real not in self.digests:
            self.digests[real] = sha(real.read_bytes())
        digest = self.digests[real]
        rules = [
            ('request', self.out / 'request'), ('config', self.out / 'codal_extra_definitions.h'),
            ('cmsis', self.out / 'cmsis'), ('device', self.args.device.resolve()), ('core', self.args.core.resolve()),
            ('board', self.args.board.resolve()), ('support', ROOT / 'no-asf-board/support'),
            ('samd', ROOT / 'inc'), ('samd', ROOT / 'src'), ('asm', ROOT / 'asm/CortexContextSwitch.s'),
            ('leaf', ROOT / 'no-asf-board/tcc-leaf'),
        ]
        need(not any(part in ('asf4', 'samd-peripherals') for part in real.parts), 'refused vendor/HAL source: ' + str(real))
        for category, base in rules:
            base = base.resolve()
            if real == base or real.is_relative_to(base):
                name = f'{category}/{real.relative_to(base).as_posix()}' if real != base else category
                break
        else:
            index = next((i for i, r in enumerate(toolchain) if real.is_relative_to(r)), None)
            need(index is not None, 'unadmitted dependency: ' + str(real))
            # Compiler and C library roots both have stdint.h: name by search-root index.
            category, name = 'toolchain', f'toolchain/{index}/' + real.relative_to(toolchain[index]).as_posix()
        need(self.admitted.get(name, digest) == digest, 'stable dependency bytes: ' + name)
        self.admitted[name] = digest
        return category

    def compile_all(self, units, flags):
        objects, failures = [], []
        obj = self.out / 'obj'
        obj.mkdir()
        def one(job):
            index, source = job
            o, d = obj / f'unit-{index}.o', obj / f'unit-{index}.d'
            language = CC if source.suffix == '.c' else CXX
            compiler = 'arm-none-eabi-gcc' if source.suffix == '.c' else 'arm-none-eabi-g++'
            argv = [compiler, *language, *COMMON, '-include', str(self.out / 'codal_extra_definitions.h'),
                    *flags, '-Wall', '-MD', '-MF', str(d), '-c', str(source), '-o', str(o)]
            p = subprocess.run(argv, cwd=ROOT, capture_output=True, timeout=600)
            (self.out / 'logs' / f'compile-{index}.stderr').write_bytes(p.stderr)
            return index, source, argv, p.returncode, o, d
        with ThreadPoolExecutor(max_workers=os.cpu_count() or 4) as pool:
            results = list(pool.map(one, enumerate(units)))
        toolchain = self.toolchain_roots()
        self.toolchain = [str(r) for r in toolchain]
        unit_rows = []
        for index, source, argv, rc, o, d in results:
            self.commands.append(dict(label=f'compile-{index}', argv=argv, returncode=rc))
            if rc:
                failures.append(str(source))
                continue
            logical = d.read_text().replace('\\\n', ' ')
            target, sep, deps = logical.partition(':')
            need(sep and '\\' not in deps, 'closed dependency rule')
            categories = sorted({self.admit(p, toolchain) for p in deps.split()})
            unit_rows.append(dict(unit=self.label(source), sha256=sha(source.read_bytes()), dependency_categories=categories))
            objects.append(o)
        need(not failures, 'compile failures: ' + ', '.join(failures))
        return objects, unit_rows

    def label(self, path):
        path = path.resolve()
        for name, base in [('request', self.out / 'request'), ('core', self.args.core), ('board', self.args.board),
                           ('device', self.args.device), ('', ROOT)]:
            base = base.resolve()
            if path.is_relative_to(base):
                return (name + ':' if name else '') + path.relative_to(base).as_posix()
        return str(path)

    def link(self, objects):
        out = self.out
        asm = out / 'obj/context-switch.o'
        self.command(['arm-none-eabi-gcc', *CPU, '-c', ROOT / 'asm/CortexContextSwitch.s', '-o', asm], 'assemble-context-switch')
        self.admit(ROOT / 'asm/CortexContextSwitch.s', [])
        script = self.args.board / 'ld/samd51g19a_flash.ld'
        elf = out / 'no-asf-board.elf'
        # The gcc driver: g++ would add libstdc++/libsupc++, which are not selected.
        self.command(['arm-none-eabi-gcc', *CPU, '-nostartfiles', '-Wl,--gc-sections', '-Wl,--no-wchar-size-warning',
                      '-Wl,-Map=' + str(out / 'no-asf-board.map'), '-T', script, '-Wl,--start-group', *objects, asm,
                      '-lm', '-lc', '-lgcc', '-Wl,--end-group', '--specs=nano.specs', '-o', elf], 'link')
        need(not self.command(['arm-none-eabi-nm', '-u', elf], 'undefined').strip(), 'closed image')
        # Archive members are dependencies too: only the C, math and compiler runtime libraries.
        members = sorted(set(re.findall(r'(lib[A-Za-z0-9_+]+\.a)\(([^)]+)\)', (out / 'no-asf-board.map').read_text())))
        archives = sorted({a for a, _ in members})
        need(set(archives) <= {'libc_nano.a', 'libm.a', 'libgcc.a'}, 'only libc_nano, libm and libgcc members: ' + ', '.join(archives))
        symbols = self.command(['arm-none-eabi-nm', '-S', '-C', elf], 'symbols').decode()
        need(not re.search(r'\b(hri_[a-z]+_d51|_sercom_get_hardware_index|hpl_|_gpio_init)\b', symbols), 'no ASF symbols')
        self.command(['arm-none-eabi-objcopy', '-O', 'binary', elf, out / 'no-asf-board.bin'], 'binary')
        self.command(['arm-none-eabi-objcopy', '-O', 'ihex', elf, out / 'no-asf-board.hex'], 'hex')
        sizes = self.command(['arm-none-eabi-size', '-A', '-x', elf], 'size').decode()
        hexlines = (out / 'no-asf-board.hex').read_text().splitlines()
        table = [i for i, line in enumerate(hexlines) if re.match(r'^:10....00' + JUMP_TABLE_MAGIC, line, re.I)]
        leaf = VARIANTS[self.args.variant].get('leaf')
        need(len(table) == (0 if leaf else 1), 'exactly one PXT function table at a 16-byte record start')
        vectors = re.search(r'^\s*([0-9a-f]+)\s+[0-9a-f]+\s+[TtRr]\s+exception_table$', symbols, re.M)
        need(vectors and int(vectors.group(1), 16) == 0x4000, 'vector table at the generated application origin 0x4000')
        return dict(elf_sha256=sha(elf.read_bytes()), bin_sha256=sha((out / 'no-asf-board.bin').read_bytes()),
                    hex_sha256=sha((out / 'no-asf-board.hex').read_bytes()), function_table_hex_line=table[0] if table else None,
                    archive_members=[f'{a}({m})' for a, m in members],
                    sections=sizes, linker_script=self.label(script))

    def run(self):
        need(not self.out.exists(), 'fresh output directory')
        self.out.mkdir(parents=True)
        (self.out / 'logs').mkdir()
        request = self.inputs()
        configuration = self.configuration()
        request_adaptation = self.adapt_request()
        colour_adaptation = self.adapt_colour()
        adaptation = self.adapt_core()
        try:
            flags = self.includes()
            units = self.units()
            objects, rows = self.compile_all(units, flags)
            image = self.link(objects)
        finally:
            subprocess.run(['git', '-C', str(self.args.core), 'checkout', '-q', '--', CORE_ADAPTATION[0]])
        counts = {}
        for name in self.admitted:
            counts[name.split('/')[0]] = counts.get(name.split('/')[0], 0) + 1
        report = dict(schema=1, status='NO_ASF_FULL_BOARD_BASE_LINKED', source=git(ROOT, 'rev-parse', 'HEAD'),
                      variant=self.args.variant, request_sha256=VARIANTS[self.args.variant]['request_sha256'],
                      request_files=len(request['replaceFiles']), core=CORE, board=BOARD,
                      atdf_sha256=ATDF_SHA256, configuration=configuration, core_adaptation=adaptation, request_adaptation=request_adaptation, colour_adaptation=colour_adaptation, units=rows,
                      inactive_units=INACTIVE, dependency_counts=counts, dependencies=self.admitted, image=image,
                      toolchain_search_roots=self.toolchain,
                      compiler=self.command(['arm-none-eabi-gcc', '--version'], 'compiler').decode().splitlines()[0],
                      guest_qualified=False, native_program=False, peripherals_qualified=False, hf2=False,
                      licence_clearance=False)
        (self.out / 'build-report.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
        (self.out / 'commands.json').write_text(json.dumps(self.commands, indent=1) + '\n')
        print(f'NO_ASF_FULL_BOARD_BASE units={len(rows)} dependencies={len(self.admitted)} {counts} elf_sha256={image["elf_sha256"]}')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ['core', 'generator', 'board', 'device', 'out']:
        p.add_argument('--' + name, type=Path, required=True)
    p.add_argument('--request', type=Path)
    p.add_argument('--colour-profile', choices=['original', 'rgb444-lut', 'short-lut'], default='original')
    p.add_argument('--timer-selection', choices=['timer-id', 'original-index'], default='timer-id')
    p.add_argument('--variant', choices=sorted(VARIANTS), default='base')
    Build(p.parse_args()).run()


if __name__ == '__main__':
    main()
