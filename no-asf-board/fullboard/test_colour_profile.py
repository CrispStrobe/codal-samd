#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Pure source-admission controls; never builds or runs a guest."""
import json
from pathlib import Path
import sys
import unittest
import build

RAW = json.loads(Path(sys.argv.pop()).read_bytes())['replaceFiles']['/pxtapp/screen---st7735/screen.cpp'].encode()


class ColourProfile(unittest.TestCase):
    def test_original_is_byte_identical(self):
        self.assertEqual(build.colour_source(RAW, 'original'), RAW)

    def test_exact_constructor_copy_and_short_control(self):
        full = build.colour_source(RAW, 'rgb444-lut')
        short = build.colour_source(RAW, 'short-lut')
        self.assertEqual(full.replace(b'n < 128;', b'n < 127;'), short)
        before, after = RAW.split(build.COLOUR_ANCHOR)
        self.assertTrue(full.startswith(before + build.COLOUR_ANCHOR))
        self.assertTrue(full.endswith(after))
        self.assertEqual(full.count(b'spi_->write(0x2d)'), 1)
        self.assertIn(b'n < 128;', full)

    def test_unadmitted_source_and_profile_refuse(self):
        for raw, profile in [(RAW + b'\n', 'rgb444-lut'), (RAW, 'invent-default')]:
            with self.assertRaises(ValueError):
                build.colour_source(raw, profile)


if __name__ == '__main__':
    unittest.main()
