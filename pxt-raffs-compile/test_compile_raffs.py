# SPDX-License-Identifier: MIT
from pathlib import Path
import unittest
from unittest import mock

import compile_raffs as raffs


class CompileControls(unittest.TestCase):
    def test_exact_two_includes_only(self):
        original = b'\n'.join(before.rstrip(b'\n') for before, _ in raffs.REPLACEMENTS) + b'\n// body\n'
        base = raffs.DECL + b'\nusing namespace pxt;\n'
        adapted, declaration = raffs.adapt(original, base)
        expected = original
        for before, after in raffs.REPLACEMENTS:
            expected = expected.replace(before, after, 1)
        self.assertEqual(adapted, expected)
        self.assertTrue(adapted.endswith(b'// body\n'))
        self.assertEqual(declaration.count(raffs.DECL), 1)
        self.assertNotIn(b'{ return', declaration)
        for raw, header in [(b'', base), (original + raffs.REPLACEMENTS[0][0], base),
                            (original, b''), (original, base + raffs.DECL + b'\n'),
                            (original, raffs.DECL + b'\n')]:
            with self.subTest(raw=raw, header=header), self.assertRaises(ValueError):
                raffs.adapt(raw, header)

    def test_closed_dependency_origins(self):
        core, out = raffs.ROOT / 'test-core', raffs.ROOT / 'test-out'
        self.assertEqual(raffs.origin(out / 'RAFFS.h', core, out),
                         ('pxt-common-packages', 'pxtapp/settings/RAFFS.h'))
        self.assertEqual(raffs.origin(out / 'pxt-hash-declaration.h', core, out),
                         ('extracted-declaration', 'pxtapp/pxtbase.h'))
        for bad in [out / 'pxtbase.h', out / 'stddef.h', out / 'pxt.h', core / 'asf4/sam.h',
                    raffs.ROOT / 'inc/platform_includes.h', raffs.ROOT.parent / 'external.h']:
            with self.subTest(path=bad), self.assertRaises(ValueError):
                raffs.origin(bad, core, out)

    def test_hosted_only_before_work(self):
        with mock.patch.dict(raffs.consumer.os.environ, {'GITHUB_ACTIONS': 'false'}), \
             mock.patch.object(raffs.subprocess, 'run') as process:
            with self.assertRaisesRegex(ValueError, 'hosted CI only'):
                raffs.run(Path('core'), Path('framework'), Path('out'))
            process.assert_not_called()


if __name__ == '__main__':
    unittest.main()
