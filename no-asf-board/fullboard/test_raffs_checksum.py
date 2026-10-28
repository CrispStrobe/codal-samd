#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 CrispStrobe contributors
"""Compile the actual CRC/entry validator and compare CRCs with Python zlib.

Native guest cases separately exercise the complete FS and atomic migration.
"""
import os
from pathlib import Path
import struct
import subprocess
import tempfile
import unittest
import zlib

from raffs_checksum import CRC_HELPERS, VALID_ENTRY


class ChecksumTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory(prefix='raffs-crc-')
        cls.addClassCleanup(cls.temporary.cleanup)
        root = Path(cls.temporary.name)
        source = root / 'crc.cpp'
        cls.binary = root / 'crc'
        source.write_bytes(b'''#include <stdint.h>
#include <string.h>
#include <stdio.h>
#include <assert.h>
#define RAFFS_MAGIC 0x7776e0db
struct FSHeader { uint32_t magic, bytes, numgc, reserved; };
struct MetaEntry { uint16_t fnhash, fnptr, _datasize, dataptr; };
''' + CRC_HELPERS + VALID_ENTRY.split(b'MetaEntry *FS::findMetaEntry')[0] + b'''
int main(int argc, char **argv) {
    if (argc == 3) {
        printf("%08x\\n", dataCRC(argv[1], argv[2], strlen(argv[2])));
        return 0;
    }
    uint8_t bank[256];
    memset(bank, 0xff, sizeof(bank));
    ((FSHeader *)bank)->magic = RAFFS_MAGIC;
    memcpy(bank + 16, "key", 4);
    memcpy(bank + 24, "data", 4);
    uint32_t crc = dataCRC("key", "data", 4);
    memcpy(bank + 28, &crc, 4);
    MetaEntry m = {0, 16, 4, 24};
    assert(validEntry(&m, bank, sizeof(bank)));
    bank[24] ^= 1;
    assert(!validEntry(&m, bank, sizeof(bank)));
    bank[24] ^= 1;
    bank[28] ^= 1;
    assert(!validEntry(&m, bank, sizeof(bank)));
    bank[28] ^= 1;
    bank[16] ^= 1;
    assert(!validEntry(&m, bank, sizeof(bank)));
    bank[16] ^= 1;
    m._datasize = 5;
    assert(!validEntry(&m, bank, sizeof(bank)));
    m._datasize = 4;
    m.dataptr = 252;
    assert(!validEntry(&m, bank, sizeof(bank)));
    m.dataptr = 0xffff;
    assert(!validEntry(&m, bank, sizeof(bank)));
    m.dataptr = 24;
    m.fnptr = 255;
    assert(!validEntry(&m, bank, sizeof(bank)));
    m.fnptr = 0;
    assert(!validEntry(&m, bank, sizeof(bank)));
    m.fnptr = 16;
    memset(bank + 24, 0xff, 8);
    assert(!validEntry(&m, bank, sizeof(bank)));
    ((FSHeader *)bank)->magic = 0x7776e0da;
    // A legacy bank has no checksum: do not reinterpret its payload as v2.
    assert(validEntry(&m, bank, sizeof(bank)));
    m.dataptr = 0;
    assert(validEntry(&m, bank, sizeof(bank)));
    return 0;
}
''')
        subprocess.run([os.environ.get('CXX', 'c++'), '-std=c++11', '-O2', str(source),
                        '-o', str(cls.binary)], check=True, capture_output=True)

    def test_crc_matches_independent_zlib_for_keys_lengths_and_payloads(self):
        for key, payload in [('key', ''), ('key', 'a'), ('other', 'a'),
                             ('key', '123456789'), ('long-key', 'payload with spaces')]:
            with self.subTest(key=key, payload=payload):
                expected = zlib.crc32(key.encode() + b'\0' + struct.pack('<I', len(payload)) + payload.encode())
                actual = subprocess.check_output([str(self.binary), key, payload], text=True).strip()
                self.assertEqual(int(actual, 16), expected)

    def test_entry_corruption_and_legacy_bounds(self):
        subprocess.run([str(self.binary)], check=True)


if __name__ == '__main__':
    unittest.main()
