# SPDX-License-Identifier: MIT
"""Pure controls only, NOT actual OOM qualification."""
from pathlib import Path
import unittest
from unittest import mock
import check_native_oom_mutants as controls

PREFIX = 'source=' + 'a' * 40 + ' elf_sha256=' + 'b' * 64 + ' step=100'


def manufactured(kind):
    if kind == 'wrong-fatal-policy':
        wrong = f'PXT_NATIVE_OOM_PANIC {PREFIX} stage=3 reason=21 raw_calls=3 raw_nulls=1 actual=0x1234'
    else:
        wrong = f'PXT_NATIVE_OOM_PAYLOAD {PREFIX} stage=3 slot=0 pointer=536875008 bytes=32 expected_sha256=' + 'c' * 64 + ' actual_sha256=' + 'd' * 64
    return (f'PXT_NATIVE_OOM_INNER_RETURN {PREFIX} stage=3 pointer=0\n' + wrong + '\n' +
        controls.MUTANTS[kind][-1] + '\ntest result: FAILED. 0 passed; 1 failed; 0 ignored;').encode()


class NativeOOMControls(unittest.TestCase):
    def test_hosted_refusal_precedes_operations(self):
        with mock.patch.dict(controls.os.environ, {'GITHUB_ACTIONS': 'false'}), mock.patch.object(controls.subprocess, 'run') as run:
            with self.assertRaisesRegex(ValueError, 'hosted CI only'): controls.main()
            run.assert_not_called()

    def test_exact_mutation_sites_refuse_ambiguity(self):
        root = Path(__file__).resolve().parents[1]
        for kind, (path, _, before, after, _) in controls.MUTANTS.items():
            original = (root / path).read_bytes() if not path.startswith('gc-link-evidence/') else before
            self.assertEqual(controls.mutate(kind, original), original.replace(before, after, 1))
            for bad in [b'', before * 2]:
                with self.assertRaises(ValueError): controls.mutate(kind, bad)

    def test_named_failure_requires_real_null_and_wrong_observation(self):
        for kind, (_, _, _, _, expected) in controls.MUTANTS.items():
            raw = manufactured(kind)
            self.assertTrue(controls.detects(raw, 101, expected))
            wrong = raw.replace(b'reason=21 ', b'reason=20 ') if kind == 'wrong-fatal-policy' else raw.replace(('d' * 64).encode(), ('c' * 64).encode())
            for bad in [wrong, raw.replace(b'INNER_RETURN', b'MISSING'), raw.replace(b'stage=3 pointer=0', b'stage=3 pointer=4'),
                raw.replace(expected.encode(), b'other failure'), raw + b'\nPXT_NATIVE_OOM_CALLER ',
                raw + b'\nPXT_NATIVE_OOM_BOUND', raw + b'\nerror: could not compile',
                raw + b'\ndid not reach expected fatal terminal within']:
                self.assertFalse(controls.detects(bad, 101, expected))
            for code in [0, 1, 124, -9, None]: self.assertFalse(controls.detects(raw, code, expected))

    def test_distinct_case_preserves_policy_and_no_public_oom_claim(self):
        root = Path(__file__).resolve().parent
        probe = (root / 'probe-native-oom.cpp').read_text()
        fixture = (root / 'guest-native-oom.rs').read_text()
        self.assertNotIn('DEVICE_PANIC_HEAP_FULL', probe)
        self.assertNotIn('gcAllocate', probe)
        self.assertIn('bw_native_realloc(old, 0x20000)', probe)
        self.assertIn('let bound = 12_000_000;', fixture)
        self.assertIn('public_gc_oom_qualified=false public_oversize_qualified=false', fixture)
        self.assertIn('0xe0000014', fixture)
        self.assertIn('only the third real raw allocation exhausts', fixture)


if __name__ == '__main__': unittest.main()
