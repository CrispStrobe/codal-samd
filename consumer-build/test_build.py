# SPDX-License-Identifier: MIT
import unittest
from unittest import mock

import build


class BuildControls(unittest.TestCase):
    def test_dependency_rule(self):
        self.assertEqual(build.dependencies('unit.o: a.h \\\n b.h a.h\n'), ['a.h', 'b.h'])

    def test_dependency_refusals(self):
        for rule in ['', 'unit.o', ': a.h', 'unit.o:', 'x.o: a.h\ny.o: b.h',
                     'x.o: escaped\\ name.h']:
            with self.subTest(rule=rule), self.assertRaises(ValueError):
                build.dependencies(rule)

    def test_source_admission(self):
        core = build.ROOT / 'owned-consumer-core'
        self.assertEqual(build.origin(core / build.SOURCE, core),
                         ('codal-core', build.PIN, build.SOURCE))
        own, pin, path = build.origin(build.ROOT / 'inc/SAMDNVM.h', core)
        self.assertEqual((own, path), ('codal-samd', 'inc/SAMDNVM.h'))
        self.assertEqual(len(pin), 40)

    def test_unknown_vendor_and_external_refuse(self):
        core = build.ROOT / 'owned-consumer-core'
        for path in [core / 'asf4/device.h', core / 'samd-peripherals/pins.h',
                     build.ROOT / 'inc/ZPin.h', build.ROOT / 'asf4/device.h',
                     build.ROOT.parent / 'compiler-header.h',
                     build.ROOT / 'runtime-services/reviewed-toolchain/not-reviewed.h']:
            with self.subTest(path=path), self.assertRaises(ValueError):
                build.origin(path, core)

    def test_hosted_entry_refuses_before_compiler(self):
        # No compiler, filesystem output or subprocess runs in these pure controls.
        with mock.patch.dict(build.os.environ, {'GITHUB_ACTIONS': 'false'}):
            with self.assertRaisesRegex(ValueError, 'hosted CI only'):
                build.hosted()


if __name__ == '__main__':
    unittest.main()
