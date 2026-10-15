# SPDX-License-Identifier: MIT
from pathlib import Path
import tempfile
import unittest

import generated


class GeneratedControls(unittest.TestCase):
    def test_original_bindings_are_not_broadened(self):
        profile = generated.sources.load(generated.sources.ROOT / 'profiles/pxt-framework-sources.json')
        unresolved = {row['request'][1:]: row for row in profile['unresolved']}
        for name, expected in generated.OUTPUTS.items():
            row = unresolved[name]
            self.assertEqual(expected, (row['git_blob'], row['sha256'], row['bytes']))
        self.assertEqual(len(unresolved), 5)  # Original census stays immutable.

    def test_changed_or_invented_generator_refuses(self):
        for raw in [b'', b'let pxtConfig = "// Configuration defines\\n"',
                    b'res.generatedFiles[sourcePath + "main.cpp"] = `invented`']:
            with self.subTest(raw=raw), self.assertRaisesRegex(ValueError, 'public generator'):
                generated.reconstruct(raw)

    def test_stage_refusals_are_atomic(self):
        with tempfile.TemporaryDirectory(prefix='pxt-generated-controls-') as temporary:
            target = Path(temporary) / 'new'
            for payloads, message in [({}, 'output census'),
                    ({name: b'invented' for name in generated.OUTPUTS}, 'component notice'),
                    ({**{name: b'' for name in generated.OUTPUTS},
                      'pxtapp/pointers.cpp': b''}, 'output census')]:
                with self.subTest(message=message), self.assertRaisesRegex(ValueError, message):
                    generated.stage(payloads, b'invalid notice', target)
                self.assertFalse(target.exists())
            with self.assertRaisesRegex(ValueError, 'new destination'):
                generated.stage({}, b'', Path(temporary))


if __name__ == '__main__':
    unittest.main()
