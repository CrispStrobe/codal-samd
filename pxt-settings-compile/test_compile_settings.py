# SPDX-License-Identifier: MIT
from pathlib import Path
import unittest
from unittest import mock
import compile_settings as settings


class CompileControls(unittest.TestCase):
    def fixture(self):
        source = b''.join(before for before,_ in settings.SETTINGS_EDITS) + b'// caller body\n'
        base = b''.join(before*count for before,count in settings.BASE_EDITS)
        return source, base + b'class PublicType { unsigned real_field; };\n// GC helper body\n'

    def test_only_declared_include_edits(self):
        source,base = self.fixture()
        adapted,header = settings.adapt(source,base)
        expected = source
        for before,after in settings.SETTINGS_EDITS:
            expected = expected.replace(before,after,1)
        self.assertEqual(adapted,expected)
        self.assertEqual(header.count(b'#include "pxt-storage-freestanding.h"\n'),7)
        self.assertTrue(header.endswith(b'class PublicType { unsigned real_field; };\n// GC helper body\n'))
        for raw,body in [(b'',base),(source,b''),(source+settings.SETTINGS_EDITS[0][0],base),
                         (source,base+settings.BASE_EDITS[0][0])]:
            with self.subTest(source=raw,base=body), self.assertRaises(ValueError):
                settings.adapt(raw,body)

    def test_exact_dependency_classes(self):
        core,out = settings.ROOT/'test-core',settings.ROOT/'test-out'
        self.assertEqual(settings.origin(out/'pxtbase-storage.h',core,out),('adapted-pxtbase',settings.BASE))
        self.assertEqual(settings.origin(out/'pxtconfig.h',core,out),('reconstructed-config','pxtapp/pxtconfig.h'))
        self.assertEqual(settings.origin(out/'pxtcore.h',core,out),('pxt-common-packages','pxtapp/pxtcore.h'))
        for path in [out/'pxt.h',out/'pxtbase.h',out/'platform.h',out/'GhostFAT.h',out/'new',
                     core/'asf4/device.h',settings.ROOT/'inc/platform_includes.h',
                     settings.ROOT.parent/'external.h']:
            with self.subTest(path=path), self.assertRaises(ValueError):
                settings.origin(path,core,out)

    def test_hosted_only_before_work(self):
        with mock.patch.dict(settings.consumer.os.environ,{'GITHUB_ACTIONS':'false'}), \
             mock.patch.object(settings.subprocess,'run') as process:
            with self.assertRaisesRegex(ValueError,'hosted CI only'):
                settings.run(Path('core'),Path('framework'),Path('generator'),Path('out'))
            process.assert_not_called()


if __name__ == '__main__':
    unittest.main()
