#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Pure mutation preparation controls; no compiler or guest execution."""
import importlib.util
from pathlib import Path
import unittest
from unittest import mock

s = importlib.util.spec_from_file_location('source_mutants', Path(__file__).with_name('check_mutants.py'))
d = importlib.util.module_from_spec(s)
s.loader.exec_module(d)


class Controls(unittest.TestCase):
    def test_named_root_panics_match_actual_status_encoding(self):
        target = (d.ROOT / 'runtime-services/target.cpp').read_text()
        self.assertIn('runtime_status = 0xe0000000u | unsigned(reason);', target)
        for kind, expected in [
            ('omit-current-fiber-roots', 'guest panic status=0xe0000a0c stage=2 checks=53'),
            ('omit-suspended-fiber-roots', 'guest panic status=0xe0000a02 stage=3 checks=58')]:
            self.assertEqual(d.MUTANTS[kind][-1], expected)
            summary = '\ntest result: FAILED. 0 passed; 1 failed; 0 ignored;'
            for wrong in [expected.replace('0xe0000', '0x'),
                          expected.replace('checks=53', 'checks=52').replace('checks=58', 'checks=57'),
                          'unrelated prefix ' + expected, expected + ' unexpected suffix']:
                self.assertFalse(d.detects((wrong + summary).encode(), 101, expected))

    def test_exact_each_mutation_and_ambiguity_refusal(self):
        for name, (_, _, before, after, _) in d.MUTANTS.items():
            raw = b'prefix\n' + before + b'suffix\n'
            self.assertEqual(d.mutate(name, raw), raw.replace(before, after))
            for bad in [b'', before * 2]:
                with self.assertRaises(ValueError):
                    d.mutate(name, bad)

    def test_named_panic_only_not_timeout_setup_or_unrelated_failure(self):
        for _, _, _, _, expected in d.MUTANTS.values():
            raw = (expected + '\ntest result: FAILED. 0 passed; 1 failed; 0 ignored;').encode()
            self.assertTrue(d.detects(raw, 101, expected))
            for code in [0, 1, -9, None]:
                self.assertFalse(d.detects(raw, code, expected))
            for extra in ['\nPXT_GC_CALLER ', '\nPXT_GC_BOUND_DIAGNOSTIC',
                          '\ndid not reach terminal within', '\nerror: could not compile']:
                self.assertFalse(d.detects(raw + extra.encode(), 101, expected))
            self.assertFalse(d.detects(b'unrelated failure', 101, expected))

    def test_no_work_before_hosted_admission(self):
        with mock.patch.dict(d.os.environ, {'GITHUB_ACTIONS': 'false'}), \
             mock.patch.object(d.subprocess, 'run') as command:
            with self.assertRaisesRegex(ValueError, 'hosted CI only'):
                d.main()
            command.assert_not_called()


if __name__ == '__main__':
    unittest.main(verbosity=2)
