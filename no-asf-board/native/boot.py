#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Assemble the flash image for a natively compiled program on the no-ASF board.

Below the application origin 0x4000 there is no bootloader code: an authored
8-byte reset forward (the program's own initial stack pointer and reset entry)
and the admitted CF2 data fragments (configuration words at 0x3000, the
information string at 0x3400 and the 16-byte UF2 information block at 0x3ff0).
The program comes unchanged from the MakeCode compiler's UF2 output.
"""
import argparse
import hashlib
import importlib.util
import json
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
APP = 0x4000
FLASH = 0x80000
FAMILY = 0x55114460  # PXT_UF2_FAMILY for SAMD51


def need(ok, reason):
    if not ok:
        raise ValueError(reason)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def program_blocks(raw):
    """UF2 payloads by target address; every block must be a SAMD51 flash block."""
    need(raw and len(raw) % 512 == 0, 'whole 512-byte UF2 blocks')
    blocks = {}
    count = len(raw) // 512
    for index in range(count):
        b = raw[512 * index:512 * index + 512]
        magic0, magic1, flags, address, size, number, total, family = struct.unpack('<8I', b[:32])
        need((magic0, magic1, struct.unpack('<I', b[508:])[0]) == (0x0A324655, 0x9E5D5157, 0x0AB16F30), 'UF2 magics')
        need(flags & 0x2000 and family == FAMILY, 'SAMD51 family block')
        need(number == index and total == count, 'complete ordered block sequence')
        need(0 < size <= 476 and APP <= address and address + size <= FLASH, 'payload inside the application region')
        need(address not in blocks, 'unique target addresses')
        blocks[address] = b[32:32 + size]
    return blocks


def assemble(uf2):
    spec = importlib.util.spec_from_file_location('cf2_metadata', ROOT / 'no-asf-board/cf2/metadata.py')
    cf2 = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(cf2)
    profile = json.loads((ROOT / 'profiles/pxt-cf2-authored-metadata.json').read_bytes())
    rows = cf2.segments(profile['words'])
    cf2.validate_segments(rows, profile['words'])
    need(profile['segments'] == [dict(address=a, bytes=len(b), sha256=sha(b)) for a, b in rows], 'admitted CF2 fragments')
    blocks = program_blocks(uf2)
    need(APP in blocks and len(blocks[APP]) >= 8, 'program vector table at the application origin')
    stack, reset = struct.unpack('<II', blocks[APP][:8])
    need(stack == 0x20030000 and reset & 1 and APP <= reset & ~1 < FLASH, 'program stack top and Thumb reset entry')
    end = max(a + len(d) for a, d in blocks.items())
    image = bytearray(b'\xff' * end)
    image[0:8] = struct.pack('<II', stack, reset)
    for address, data in rows:
        image[address:address + len(data)] = data
    for address, data in sorted(blocks.items()):
        image[address:address + len(data)] = data
    return bytes(image), dict(reset_forward=dict(stack=stack, reset=reset), cf2_fragments=profile['segments'],
                              program_blocks=len(blocks), program_end=end, bootloader_code=False)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('uf2', type=Path)
    p.add_argument('out', type=Path)
    a = p.parse_args()
    raw = a.uf2.read_bytes()
    image, report = assemble(raw)
    a.out.write_bytes(image)
    report.update(uf2_sha256=sha(raw), image_sha256=sha(image), image_bytes=len(image))
    a.out.with_suffix('.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(f'NO_ASF_NATIVE_IMAGE bytes={len(image)} image_sha256={report["image_sha256"]} blocks={report["program_blocks"]}')


if __name__ == '__main__':
    main()
