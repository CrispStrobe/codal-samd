#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Reconstruct two fixed PXT inputs from a pinned public generator, not a build."""
import argparse
import json
from pathlib import Path
import re

import sources

PIN = 'c40f412dc5bebb915a5891cd62fe956e198fb194'
TREE = '3e5a442c4265eaf2e5b2495eb68539b86841716e'
GENERATOR = 'e2d5c77c6f67c8f93c4efee4cdf098a2fdde1ff9'
NOTICE = 'd859446c6968eaeff058d8c5de28e607ffb4c717'
OUTPUTS = {
    'pxtapp/pxtconfig.h': ('82b1e37da518f53d422629731d87860a3f653e74',
        'c5cbda6b4a6a0ae26be260d587b0181740a8e32eebe6e8a47679659df7037358', 78),
    'pxtapp/main.cpp': ('b37fbff61929a64845dbf2cb77a26cc88d6011d4',
        'e9f2fef733dbd4caa20393d4e19f022a5bc3317f7dedff88838d291945d91114', 187),
}


def reconstruct(generator):
    sources.need(sources.identity(generator)[0] == GENERATOR, 'changed public generator')
    text = generator.decode('utf-8')
    # Fixed, deliberately narrow reconstruction. No TS execution, package/shim
    # scanner, interpolation evaluator or user-supplied expression is admitted.
    preamble = re.findall(r'let pxtConfig = ("[^\n]*")', text)
    sources.need(len(preamble) == 1, 'configuration preamble census')
    config = json.loads(preamble[0])
    sources.need('if (compile.utf8)\n            cpp_options["PXT_UTF8"] = 1' in text,
                 'UTF8 generator rule')
    sources.need('pxtConfig += `#define ${k} ${cpp_options[k]}\\n`' in text,
                 'option generator rule')
    sources.need('pxtConfig += `#define PXT_UF2_FAMILY ${compile.uf2Family}\\n`' in text,
                 'UF2 generator rule')
    # These selected values reproduce the original two files. They do not assert
    # a complete historical compile-options object or a new device configuration.
    config += '#define PXT_UTF8 1\n#define PXT_UF2_FAMILY 0x55114460\n'
    main = re.findall(r'res\.generatedFiles\[sourcePath \+ "main\.cpp"\] = `([^`]*)`', text)
    sources.need(len(main) == 1 and '${' not in main[0], 'literal main template census')
    payloads = {'pxtapp/pxtconfig.h': config.encode(), 'pxtapp/main.cpp': main[0].encode()}
    for name, expected in OUTPUTS.items():
        sources.need(sources.identity(payloads[name]) == expected, 'original input byte identity')
    return payloads


def verify(checkout):
    checkout = checkout.resolve()
    sources.need(sources.git(checkout, 'rev-parse', 'HEAD').decode().strip() == PIN,
                 'wrong generator commit')
    sources.need(sources.git(checkout, 'rev-parse', 'HEAD^{tree}').decode().strip() == TREE,
                 'wrong generator tree')
    sources.need(not sources.git(checkout, 'status', '--porcelain', '--untracked-files=no'),
                 'dirty generator checkout')
    data = {}
    for name, blob in [('pxtlib/cpp.ts', GENERATOR), ('LICENSE', NOTICE)]:
        path = checkout / name
        sources.need(not path.is_symlink() and path.resolve().is_relative_to(checkout),
                     'generator input escape')
        raw = path.read_bytes()
        sources.need(sources.identity(raw)[0] == blob and
                     raw == sources.git(checkout, 'show', PIN + ':' + name),
                     'changed generator input/notice')
        data[name] = raw
    return reconstruct(data['pxtlib/cpp.ts']), data['LICENSE']


def stage(payloads, notice, destination):
    sources.need(not destination.exists() and not destination.is_symlink(),
                 'owned new destination required')
    sources.need(set(payloads) == set(OUTPUTS), 'generated output census')
    sources.need(sources.identity(notice)[0] == NOTICE, 'generator component notice')
    for name, expected in OUTPUTS.items():
        sources.need(sources.identity(payloads[name]) == expected, 'changed generated output')
    # Validate everything before any output; never overlay or admit the shim table.
    destination.mkdir(parents=True)
    (destination / 'pxtapp').mkdir()
    for name, raw in payloads.items():
        (destination / name).write_bytes(raw)
    (destination / 'LICENSE-pxt-generator').write_bytes(notice)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verify-checkout', required=True, type=Path)
    parser.add_argument('--stage', type=Path)
    args = parser.parse_args()
    payloads, notice = verify(args.verify_checkout)
    if args.stage:
        stage(payloads, notice, args.stage)
    print(json.dumps(dict(status='TWO_FIXED_GENERATED_INPUTS_RECONSTRUCTED',
        original_byte_matches=2, typescript_generator_executed=False,
        full_generated_inputs_qualified=False, consumer_compiled=False,
        image_admitted=False, licence_clearance=False), sort_keys=True))
