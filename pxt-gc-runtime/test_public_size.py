# SPDX-License-Identifier: MIT
"""Manufactured matcher/admission controls, not actual guest results."""
import unittest
from unittest import mock
import check_public_size_mutants as controls

PREFIX = 'source=' + 'a' * 40 + ' elf_sha256=' + 'b' * 64 + ' step=100'


def manufactured(kind):
    if kind == 'wrong-size-cap-panic':
        entry = f'PUBLIC_SIZE_ENTRY {PREFIX} stage=6 index=3 request=131076 actual=0x1234'
        wrong = f'PUBLIC_SIZE_PANIC {PREFIX} stage=6 reason=21'
    else:
        entry = f'PUBLIC_SIZE_ENTRY {PREFIX} stage=2 index=0 request=32 actual=0x1234'
        wrong = f'PUBLIC_SIZE_HEADER {PREFIX} stage=2 index=1 pointer=536875008 header=0x40000024 expected=0x60000024'
    return (entry + '\n' + wrong + '\n' + controls.MUTANTS[kind][-1] +
            '\ntest result: FAILED. 0 passed; 1 failed; 0 ignored;').encode()


class PublicSizeControls(unittest.TestCase):
    def test_hosted_guard_precedes_files_and_processes(self):
        for env in [dict(GITHUB_ACTIONS='false', RUNNER_ENVIRONMENT='github-hosted'),
                    dict(GITHUB_ACTIONS='true', RUNNER_ENVIRONMENT='self-hosted')]:
            with mock.patch.dict(controls.os.environ, env), \
                 mock.patch.object(controls.subprocess, 'run') as run, \
                 mock.patch.object(controls.subprocess, 'check_output') as output, \
                 mock.patch.object(controls.Path, 'exists') as exists:
                with self.assertRaisesRegex(ValueError, 'hosted CI only'):
                    controls.main()
                run.assert_not_called()
                output.assert_not_called()
                exists.assert_not_called()

    def test_unique_exact_mutation_sites(self):
        for kind, (_, _, before, after, _) in controls.MUTANTS.items():
            self.assertEqual(controls.mutate(kind, b'prefix\n' + before + b'\nsuffix'),
                             b'prefix\n' + after + b'\nsuffix')
            for bad in [b'', before * 2, before.replace(b';', b' ;')]:
                with self.assertRaisesRegex(ValueError, 'unique exact original mutation site'):
                    controls.mutate(kind, bad)

    def test_named_actual_wrong_observation_required(self):
        for kind, (_, _, _, _, expected) in controls.MUTANTS.items():
            raw = manufactured(kind)
            self.assertTrue(controls.detects(raw, 101, expected))
            wrong = (raw.replace(b'reason=21', b'reason=22') if kind == 'wrong-size-cap-panic'
                     else raw.replace(b'header=0x40000024', b'header=0x60000024'))
            for bad in [wrong, raw.replace(b'PUBLIC_SIZE_ENTRY', b'MISSING'),
                        raw.replace(b'stage=6', b'stage=5'),
                        raw.replace(b'stage=2', b'stage=3'),
                        raw.replace(expected.encode(), b'other assertion'),
                        raw.replace(b'1 failed', b'0 failed'), raw + raw,
                        raw + b'\nPUBLIC_SIZE_DIAGNOSTIC ', raw + b'\nPUBLIC_SIZE_BOUND',
                        raw + b'\nPUBLIC_SIZE_BOUND source=x',
                        raw + b'\ndid not reach its named terminal within',
                        raw + b'\nerror: could not compile', b'\xff']:
                if bad != raw:
                    self.assertFalse(controls.detects(bad, 101, expected))
            for code in [0, 1, 124, -9, None]:
                self.assertFalse(controls.detects(raw, code, expected))
            self.assertFalse(controls.detects(raw, 101, 'unrecognized assertion'))
            self.assertTrue(controls.detects(raw + f'\nPUBLIC_SIZE_BOUNDARY {PREFIX} stage=5'.encode(), 101, expected))

    def test_exact_wrong_arguments_and_nonnull_pointer(self):
        for kind, before, after in [
            ('wrong-size-cap-panic', b'request=131076', b'request=131072'),
            ('wrong-size-cap-panic', b'index=3', b'index=2'),
            ('omit-size-permanent-ownership', b'pointer=536875008', b'pointer=0'),
            ('omit-size-permanent-ownership', b'index=1', b'index=0'),
        ]:
            self.assertFalse(controls.detects(manufactured(kind).replace(before, after), 101,
                                             controls.MUTANTS[kind][-1]))


if __name__ == '__main__':
    unittest.main()
