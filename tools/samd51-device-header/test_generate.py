#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Layout controls for the generated SAMD51 device headers.

Every register of every generated struct is checked against its ATDF offset and
width by compiling static assertions with the selected ARM compiler. A perturbed
header must fail the same check, a changed ATDF must be refused, and the USB
endpoint/descriptor layout of the independently authored USB leaf header must
agree with the ATDF-derived facts.
"""
import importlib.util
import re
import shutil
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('generate', Path(__file__).with_name('generate.py'))
gen = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gen)


def need(ok, reason):
    if not ok:
        raise SystemExit('SAMD51_DEVICE_HEADER_CONTROL FAIL: ' + reason)


def assertions(g):
    """(struct, member, offset, member bytes) for every generated struct member."""
    rows = []

    def walk(module, group, mode, name):
        for item in g.group_members(module, group, mode):
            offset = gen.num(item.get('offset'))
            count = gen.num(item.get('count')) if item.get('count') else None
            if item.tag == 'register':
                size = gen.num(item.get('size'))
                rows.append((name, item.get('name'), offset, size, count))
            else:
                sub = g.subgroup(module, item.get('name-in-module'))
                subname = gen.camel(module) + gen.camel(item.get('name-in-module'))
                walk(module, sub, mode if sub.find('mode') is not None else None, subname)
                rows.append((name, gen.camel(item.get('name')), offset, gen.num(sub.get('size')), count))

    for module in g.modules:
        if module in gen.SKIPPED_MODULES:
            continue
        for group in g.modules[module].findall('register-group'):
            modes = [m.get('name') for m in group.findall('mode') if (module, m.get('name')) not in gen.DROPPED_MODES]
            if group.get('name') == module:
                for mode in modes or [None]:
                    walk(module, group, mode, gen.camel(module) + (gen.camel(g.mode_name(module, mode)) if mode else ''))
            elif modes and any(x.tag == 'register-group' for x in group):
                suffix = gen.camel(group.get('name').replace(module + '_', '', 1))
                for mode in modes:
                    walk(module, group, mode, gen.camel(module) + gen.camel(mode) + suffix)
    return rows


def source(rows, include, only=None):
    lines = ['#include <stddef.h>', f'#include "{include}"']
    for i, (struct, member, offset, size, count) in enumerate(rows):
        if only and struct not in only:
            continue
        lines.append(f'static_assert(offsetof({struct}, {member}) == {offset:#x}, "{struct}.{member} offset");')
        lines.append(f'static_assert(sizeof((({struct} *)0)->{member}) == {size * (count or 1)}, "{struct}.{member} width");')
    return '\n'.join(lines) + '\n'


def compile_check(text, includes, defines=()):
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / 'check.cpp'
        path.write_text(text)
        argv = ['arm-none-eabi-g++', '-std=c++11', '-mcpu=cortex-m4', '-mthumb', '-fsyntax-only', '-Wno-invalid-offsetof',
                *[f'-D{d}' for d in defines], *sum((['-I', str(i)] for i in includes), []), str(path)]
        return subprocess.run(argv, capture_output=True, text=True)


# Leaf bitfield unions and the generated field each one describes.
LEAF_FIELDS = {
    'BwUsbControl': {'SWRST': 'USB_CTRLA_SWRST_Pos', 'ENABLE': 'USB_CTRLA_ENABLE_Pos',
                     'RUNSTDBY': 'USB_CTRLA_RUNSTDBY_Pos', 'MODE': 'USB_CTRLA_MODE_Pos'},
    'BwUsbSync': {'SWRST': 'USB_SYNCBUSY_SWRST_Pos', 'ENABLE': 'USB_SYNCBUSY_ENABLE_Pos'},
    'BwUsbDeviceControl': {'DETACH': 'USB_DEVICE_CTRLB_DETACH_Pos', 'UPRSM': 'USB_DEVICE_CTRLB_UPRSM_Pos',
                           'SPDCONF': 'USB_DEVICE_CTRLB_SPDCONF_Pos'},
    'BwUsbPadCalibration': {'TRANSP': 'USB_PADCAL_TRANSP_Pos', 'TRANSN': 'USB_PADCAL_TRANSN_Pos',
                            'TRIM': 'USB_PADCAL_TRIM_Pos'},
    'BwUsbPacketSize': {'BYTE_COUNT': 'USB_DEVICE_PCKSIZE_BYTE_COUNT_Pos',
                        'MULTI_PACKET_SIZE': 'USB_DEVICE_PCKSIZE_MULTI_PACKET_SIZE_Pos',
                        'SIZE': 'USB_DEVICE_PCKSIZE_SIZE_Pos', 'AUTO_ZLP': 'USB_DEVICE_PCKSIZE_AUTO_ZLP_Pos'},
    'BwUsbPinConfig': {'PMUXEN': 'PORT_PINCFG_PMUXEN_Pos'},
    'BwUsbGclkSync': {'GENCTRL0': 'GCLK_SYNCBUSY_GENCTRL_Pos'},
    'BwUsbAhbMask': {'USB_': 'MCLK_AHBMASK_USB_Pos'},
    'BwUsbApbbMask': {'USB_': 'MCLK_APBBMASK_USB_Pos'},
}


def leaf_crosscheck(device):
    """Bit positions and numeric facts of the hand-authored USB leaf header vs the ATDF."""
    header = ROOT / 'no-asf-board/usb/samd51_usb_mmio.h'
    with tempfile.TemporaryDirectory() as tmp:
        probe = Path(tmp) / 'probe.cpp'
        lines = ['#define BW_USB_LAYOUT_CONTROL 1', '#include <stdio.h>', f'#include "{header}"', 'int main() {']
        for union, fields in LEAF_FIELDS.items():
            for field in fields:
                lines.append(f'  {{ {union} u; u.reg = 0; u.bit.{field} = 1; printf("{union} {field} %d\\n", __builtin_ctzll((unsigned long long)u.reg)); }}')
        lines.append('}')
        probe.write_text('\n'.join(lines) + '\n')
        exe = Path(tmp) / 'probe'
        built = subprocess.run(['c++', '-std=c++11', str(probe), '-o', str(exe)], capture_output=True, text=True)
        need(built.returncode == 0, 'leaf bitfield probe builds:\n' + built.stderr[:2000])
        positions = {tuple(l.split()[:2]): int(l.split()[2]) for l in subprocess.check_output([str(exe)], text=True).splitlines()}
    checks = [f'static_assert({LEAF_FIELDS[u][f]} == {p}, "leaf {u}.{f} bit {p}");' for (u, f), p in sorted(positions.items())]
    literal = re.compile(r'^#define ([A-Z][A-Z0-9_]*) (0x[0-9a-fA-F]+u?|[0-9]+u?)$', re.M)
    values = literal.findall(header.read_text())
    checks += [f'#ifdef {n}\nstatic_assert(({n}) == ({v}), "leaf {n}");\n#endif' for n, v in values]
    result = compile_check(f'#include "{gen.DEVICE.lower()}.h"\n#include "fuses_compat.h"\n' + '\n'.join(checks) + '\n',
                           [device, ROOT / 'inc/CMSIS', ROOT / 'no-asf-board/support'])
    need(result.returncode == 0, 'USB leaf header disagrees with the ATDF:\n' + result.stderr[:4000])
    return len(positions), len(values)


def main():
    atdf, device = Path(sys.argv[1]), Path(sys.argv[2])
    raw = atdf.read_bytes()
    g = gen.Generator(ET.fromstring(raw))
    rows = assertions(g)
    need(len(rows) > 500, f'nonvacuous layout census ({len(rows)} members)')
    includes = [device, ROOT / 'inc/CMSIS']
    result = compile_check(source(rows, gen.DEVICE.lower() + '.h'), includes)
    need(result.returncode == 0, 'generated layout differs from ATDF:\n' + result.stderr[:4000])

    # A single reserved byte inserted into TcCount16 must be detected.
    with tempfile.TemporaryDirectory() as tmp:
        perturbed = Path(tmp) / 'device'
        shutil.copytree(device, perturbed)
        tc = perturbed / 'component/tc.h'
        text = tc.read_text()
        marker = '  __IO TC_CTRLA_Type CTRLA;\n'
        index = text.index('} TcCount16;')
        start = text.rindex(marker, 0, index)
        tc.write_text(text[:start] + marker + '       RoReg8 BwPerturbation;\n' + text[start + len(marker):])
        result = compile_check(source(rows, gen.DEVICE.lower() + '.h', only={'TcCount16'}), [perturbed, ROOT / 'inc/CMSIS'])
        need(result.returncode != 0 and 'TcCount16.CTRLBCLR offset' in result.stderr, 'perturbed layout detected')

    # The generator refuses any ATDF other than the admitted bytes.
    with tempfile.TemporaryDirectory() as tmp:
        changed = Path(tmp) / 'changed.atdf'
        changed.write_bytes(raw.replace(b'offset="0x40003800"', b'offset="0x40003804"', 1))
        p = subprocess.run([sys.executable, '-B', str(Path(__file__).with_name('generate.py')), str(changed), str(Path(tmp) / 'out')],
                           capture_output=True, text=True)
        need(p.returncode != 0 and 'not the admitted input' in p.stderr, 'changed ATDF refused')

    # Independent cross-check: the hand-authored USB leaf header against the ATDF.
    usb = {'UsbDeviceEndpoint', 'UsbDeviceDescBank'}
    leaf_rows = [r for r in rows if r[0] in usb]
    leaf = source(leaf_rows, 'samd51_usb_mmio.h').replace('UsbDeviceDescBank', 'BwUsbBank')
    result = compile_check(leaf, [ROOT / 'no-asf-board/usb'], defines=['BW_USB_LAYOUT_CONTROL'])
    need(result.returncode == 0, 'USB leaf header disagrees with the ATDF:\n' + result.stderr[:4000])
    fields, macros = leaf_crosscheck(device)
    print(f'SAMD51_DEVICE_HEADER_CONTROL PASS leaf_bitfields={fields} leaf_literals={macros}'
          f' members={len(rows)} perturbation_detected=true changed_atdf_refused=true '
          f'usb_leaf_crosscheck={len(leaf_rows)}')


if __name__ == '__main__':
    main()
