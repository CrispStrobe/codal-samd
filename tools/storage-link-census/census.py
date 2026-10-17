#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Pinned public-source storage gap census; never compiles or executes a guest."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/pxt-storage'))
import sources

ENGINE = '73f8475f8e406b1ff4475bc9886e22479881d67a'
DESCRIPTOR = 'configs/chips/atsamd51-pybadge.yaml'
OWN = ['profiles/pxt-framework-sources.json', 'profiles/historical-board-consumers.json',
       'pxt-storage-compile/pxt-flash-platform.h', 'runtime-services/platform_includes.h',
       'docs/receipts/2026-10-15-pxt-flash-compile.json',
       'docs/receipts/2026-10-15-pxt-raffs-compile.json',
       'docs/receipts/2026-10-15-pxt-settings-compile.json']
WITNESSES = {
    'pxtapp/settings/SAMDFlash.cpp': {
        'ready_poll': 'while (NVMCTRL->STATUS.bit.READY == 0)',
        'geometry': 'return (8 << NVMCTRL->PARAM.bit.PSZ) * NVMCTRL->PARAM.bit.NVMP;',
        'erase_command': 'CMD(NVMCTRL_CTRLA_CMD_ER, NVMCTRL_CTRLB_CMD_EB);',
        'buffer_clear': 'CMD(NVMCTRL_CTRLA_CMD_PBC, NVMCTRL_CTRLB_CMD_PBC);',
        'quadword_program': 'CMD(NVMCTRL_CTRLA_CMD_WP, NVMCTRL_CTRLB_CMD_WQW);',
        'flash_store': '*dp = v;',
        'cache_status': 'while (CMCC->SR.bit.CSTS)',
        'cache_invalidate': 'CMCC->MAINT0.bit.INVALL = 1;',
        'eight_byte_length': 'if (len & 7)',
        'eight_byte_destination': 'if (dst & 7)',
        'sixteen_byte_window': '#define WRITE_SIZE 16',
    },
    'pxtapp/settings/settings.cpp': {
        'default_storage_bytes': '#define SETTINGS_SIZE (32 * 1024)',
        'samd51_storage_range': 'fs(flash, 512 * 1024 - size, size),',
        'config_default': 'getConfig(CFG_SETTINGS_SIZE_DEFL, SETTINGS_SIZE)',
        'config_override': 'getConfig(CFG_SETTINGS_SIZE, 0)',
        'mount': 's->fs.tryMount()',
        'write': 's->fs.write(key->getUTF8Data(), data->data, data->length)',
        'read': 's->fs.read(NULL, ret->data, ret->length)',
        'buffer_allocation': 'mkBuffer(NULL, sz)',
    },
    'pxtapp/platform.h': {
        name: '#include "' + name + '"' for name in
        ['ZPin.h', 'ZSPI.h', 'ZI2C.h', 'SAMDNVM.h', 'SAMDDAC.h', 'SAMDPDM.h', 'SAMDSerial.h']
    },
}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def public_payloads(stage, profile):
    sources.validate(profile)
    sources.need(sources.identity((stage / 'LICENSE-pxt-common-packages').read_bytes())[0]
                 == sources.NOTICE, 'framework notice identity')
    result = {}
    for row in profile['sources']:
        name = row['request'].lstrip('/')
        path = stage / name
        sources.need(not path.is_symlink() and path.resolve().is_relative_to(stage.resolve()),
                     'public source escape or symlink')
        raw = path.read_bytes()
        sources.need(sources.identity(raw) == (row['git_blob'], row['sha256'], row['bytes']),
                     'public source byte identity')
        result[name] = raw
    return result


def source_observations(payloads):
    result = {}
    for path, observations in WITNESSES.items():
        text = payloads[path].decode('utf-8')
        lines = text.splitlines()
        found = {}
        for name, token in observations.items():
            hits = [i + 1 for i, line in enumerate(lines) if token in line]
            sources.need(bool(hits), 'missing source witness: ' + name)
            found[name] = hits
        result[path] = dict(sha256=sha(payloads[path]), source_lines=found)
    return result


def descriptor_observations(raw):
    # This is a deliberately narrow review of this exact descriptor, not a YAML
    # resolver or evidence that no other engine subsystem could implement NVM.
    text = '\n'.join(line.split('#', 1)[0].rstrip() for line in raw.decode().splitlines())
    sources.need(text.count('\nperipherals:') == 1 and 'include:' not in text,
                 'standalone descriptor required')
    tail = text.split('\nperipherals:', 1)[1]
    entries = []
    for block in re.split(r'\n  - id: ', tail)[1:]:
        name = re.match(r'"([a-z0-9]+)"\n', block)
        kind = re.search(r'^    type: "([a-z0-9_]+)"$', block, re.M)
        base = re.search(r'^    base_address: (0x[0-9A-Fa-f]+)$', block, re.M)
        sources.need(name is not None and kind is not None and base is not None,
                     'unsupported peripheral entry')
        entries.append(dict(id=name[1], type=kind[1], address=int(base[1], 16)))
    sources.need(bool(entries) and len(entries) == len(re.findall(r'^\s*-\s+id:', tail, re.M))
                 and len({r['id'] for r in entries}) == len(entries),
                 'missing or duplicate peripheral identity')
    missing = [dict(id=name, address=address) for name, address in
               [('nvmctrl', 0x41004000), ('cmcc', 0x41006000)]
               if not any(row['address'] == address for row in entries)]
    return dict(entries=entries, required_addresses_not_declared=missing)


def build(stage, engine):
    profile = sources.load(ROOT / OWN[0])
    payloads = public_payloads(stage, profile)
    raw = subprocess.check_output(['git', '-C', str(engine), 'show', ENGINE + ':' + DESCRIPTOR])
    result = dict(schema=1, status='PINNED_STORAGE_SOURCE_GAP_CENSUS_ONLY',
                  framework=profile['package'], public_source_files=len(payloads),
                  source_observations=source_observations(payloads),
                  engine=dict(pin=ENGINE, path=DESCRIPTOR, sha256=sha(raw),
                              descriptor=descriptor_observations(raw)),
                  own_inputs=[dict(path=p, sha256=sha((ROOT / p).read_bytes())) for p in OWN],
                  compiled=False, linked=False, executed=False, storage_qualified=False,
                  original_board_configuration_qualified=False, image_admitted=False,
                  licence_clearance=False)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', type=Path)
    parser.add_argument('engine', type=Path)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--output', type=Path)
    group.add_argument('--check', type=Path)
    args = parser.parse_args()
    raw = (json.dumps(build(args.stage, args.engine), sort_keys=True, indent=2) + '\n').encode()
    if args.output:
        with args.output.open('xb') as file:
            file.write(raw)
    else:
        sources.need(args.check.read_bytes() == raw, 'committed source census differs')
    print('STORAGE_SOURCE_CENSUS_ONLY public_sources=64; no compiler/link/guest/image admission')


if __name__ == '__main__':
    main()
