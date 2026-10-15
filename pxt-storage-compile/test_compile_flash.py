# SPDX-License-Identifier: MIT
from pathlib import Path
import unittest
from unittest import mock

import compile_flash as flash


class CompileControls(unittest.TestCase):
    def test_adaptation_changes_only_the_selected_include(self):
        raw = flash.ADAPT_FROM + b'#include "Flash.h"\n// driver body\n'
        result = flash.adapt(raw)
        self.assertEqual(result, flash.ADAPT_TO + raw[len(flash.ADAPT_FROM):])
        for bad in [b'', b'#include "other.h"\n', b'\n' + raw, raw + flash.ADAPT_FROM]:
            with self.subTest(raw=bad), self.assertRaisesRegex(ValueError, 'unique original'):
                flash.adapt(bad)

    def test_closed_dependency_origins(self):
        core, out = flash.ROOT / 'test-core', flash.ROOT / 'test-out'
        self.assertEqual(flash.origin(out / 'Flash.h', core, out),
                         ('pxt-common-packages', flash.FLASH))
        self.assertEqual(flash.origin(out / 'SAMDFlash-storage.cpp', core, out),
                         ('adapted-driver', flash.DRIVER))
        for name in flash.OWN:
            self.assertEqual(flash.origin(flash.ROOT / name, core, out), ('codal-samd', name))
        for bad in [core / 'asf4/sam.h', flash.ROOT / 'inc/platform_includes.h',
                    out / 'pxt.h', out / 'sam.h', flash.ROOT.parent / 'external.h']:
            with self.subTest(path=bad), self.assertRaises(ValueError):
                flash.origin(bad, core, out)

    def test_entry_is_hosted_only_before_any_subprocess(self):
        with mock.patch.dict(flash.consumer.os.environ, {'GITHUB_ACTIONS': 'false'}), \
             mock.patch.object(flash.subprocess, 'run') as process:
            with self.assertRaisesRegex(ValueError, 'hosted CI only'):
                flash.run(Path('core'), Path('framework'), Path('out'))
            process.assert_not_called()


if __name__ == '__main__':
    unittest.main()
