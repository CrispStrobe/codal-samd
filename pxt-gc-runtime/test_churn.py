# SPDX-License-Identifier: MIT
"""Pure source/controller controls, not allocation or failure qualification."""
from pathlib import Path
import unittest
from unittest import mock
import check_churn_mutants as controls

PREFIX = 'source=' + 'a' * 40 + ' elf_sha256=' + 'b' * 64 + ' step=100'


def manufactured(expected):
    if expected == 'native arithmetic refusal must return null':
        line = f'PXT_GC_CHURN_RETURN {PREFIX} sequence=2 op=7 slot=8 bytes=4294967295 old=0 pointer=536875008 guest_inputs_unchanged=true'
    else:
        line = f'PXT_GC_CHURN_COPY {PREFIX} sequence=21 bytes=1 old=536875008 pointer=536879104 expected_sha256=' + 'c' * 64 + ' actual_sha256=' + 'd' * 64 + ' guest_inputs_unchanged=true'
    return (line + '\n' + expected + '\ntest result: FAILED. 0 passed; 1 failed; 0 ignored;').encode()


class ChurnControls(unittest.TestCase):
    def test_valid_boundary_prefix_is_not_a_timeout_marker(self):
        boundary = (f'PXT_GC_CHURN_BOUNDARY {PREFIX} sequence=1 op=6 slot=8 bytes=0 '
                    'pointer=0 checks=2 live=0 contents_verified=true guest_inputs_unchanged=true\n').encode()
        for _, _, _, _, expected in controls.MUTANTS.values():
            raw = boundary + manufactured(expected)
            self.assertTrue(controls.detects(raw, 101, expected))
            for suffix in [b'\nPXT_GC_CHURN_BOUND',
                           ('\nPXT_GC_CHURN_BOUND ' + PREFIX + ' bound=12000000 qualified=false').encode()]:
                self.assertFalse(controls.detects(raw + suffix, 101, expected))

    def test_mutants_change_only_one_actual_native_site(self):
        original = (Path(__file__).resolve().parents[1] / 'pxt-native-allocation/native-aligned-heap.cpp').read_bytes()
        for kind, (_, _, before, after, _) in controls.MUTANTS.items():
            self.assertEqual(original.count(before), 1)
            self.assertEqual(controls.mutate(kind, original), original.replace(before, after, 1))
            for bad in [b'', before * 2]:
                with self.assertRaises(ValueError):
                    controls.mutate(kind, bad)

    def test_named_failure_requires_wrong_real_observation(self):
        for _, _, _, _, expected in controls.MUTANTS.values():
            raw = manufactured(expected)
            self.assertTrue(controls.detects(raw, 101, expected))
            for bad in [raw.replace(b'PXT_GC_CHURN_', b'MISSING_'),
                        raw.replace(expected.encode(), b'other failure'),
                        raw.replace(b'pointer=536875008 guest', b'pointer=0 guest') if 'arithmetic' in expected else
                        raw.replace(('d' * 64).encode(), ('c' * 64).encode()),
                        raw + b'\nPXT_GC_CHURN_CALLER ', raw + b'\nPXT_GC_CHURN_BOUND',
                        raw + b'\nerror: could not compile', raw + b'\ndid not reach terminal within']:
                self.assertFalse(controls.detects(bad, 101, expected))
            for code in [0, 1, 124, -9, None]:
                self.assertFalse(controls.detects(raw, code, expected))

    def test_host_refusal_precedes_any_command(self):
        with mock.patch.dict(controls.os.environ, {'GITHUB_ACTIONS': 'false'}), \
             mock.patch.object(controls.subprocess, 'run') as command:
            with self.assertRaisesRegex(ValueError, 'hosted CI only'):
                controls.main()
            command.assert_not_called()

    def test_declared_finite_counts_and_no_false_oom_claim(self):
        root = Path(__file__).resolve().parent
        probe = (root / 'probe-churn.cpp').read_text()
        fixture = (root / 'guest-churn.rs').read_text()
        # Per round: alloc8*2, refusal4*2, grow8*3, shrink8*3,
        # two actual GC boundaries and eight post-free checks.
        self.assertEqual(8 * (16 + 8 + 24 + 24 + 2 + 8) + 5 + 1, 662)
        self.assertEqual(8 * (8 + 4 + 8 + 8 + 2 + 8) + 4, 308)
        self.assertIn('churn_checks == 661 && churn_sequence == 308 && churn_boundaries == 308', probe)
        self.assertIn('let bound = 12_000_000;', fixture)
        self.assertIn('real_oom_qualified=false allocated_dma_qualified=false', fixture)
        self.assertIn('actual API return must precede each boundary', fixture)
        self.assertIn('every live payload byte after disturbance', fixture)
        self.assertIn('expected_sha256=', fixture)
        self.assertIn('gc(0);', probe)
        self.assertNotIn('DEVICE_PANIC_HEAP_FULL', probe)


if __name__ == '__main__':
    unittest.main()
