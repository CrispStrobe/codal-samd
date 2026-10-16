# SPDX-License-Identifier: MIT
"""Manufactured controller controls; no compiler or guest execution."""
from pathlib import Path
import unittest
from unittest import mock
import check_public_oom_mutants as controls

PREFIX = 'source=' + 'a' * 40 + ' elf_sha256=' + 'b' * 64 + ' step=100'


def manufactured(kind):
    if kind == 'wrong-public-oom-policy':
        wrong = f'PUBLIC_GC_OOM_PANIC {PREFIX} attempt=19 completed=18 reason=22 block_calls=2 raw_calls=1 collections=5 actual=0x1234'
    else:
        wrong = f'PUBLIC_GC_OOM_OWNERSHIP {PREFIX} attempt=1 pointer=536875008 header=0x40002004 expected=0x60002004'
    return (f'PUBLIC_GC_OOM_RAW_RETURN {PREFIX} pointer=536875000 capacity=147628 heap_bytes=180404\n' +
            wrong + '\n' + controls.MUTANTS[kind][-1] +
            '\ntest result: FAILED. 0 passed; 1 failed; 0 ignored;').encode()


class PublicOOMControls(unittest.TestCase):
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

    def test_unique_exact_source_sites(self):
        for kind, (_, _, before, after, _) in controls.MUTANTS.items():
            original = b'prefix\n' + before + b'\nsuffix'
            self.assertEqual(controls.mutate(kind, original), b'prefix\n' + after + b'\nsuffix')
            for bad in [b'', before * 2, before.replace(b';', b' ;')]:
                with self.assertRaisesRegex(ValueError, 'unique exact original mutation site'):
                    controls.mutate(kind, bad)

    def test_named_wrong_observation_and_actual_backing_return_required(self):
        for kind, (_, _, _, _, expected) in controls.MUTANTS.items():
            raw = manufactured(kind)
            self.assertTrue(controls.detects(raw, 101, expected))
            wrong = (raw.replace(b'reason=22 ', b'reason=21 ') if kind == 'wrong-public-oom-policy'
                     else raw.replace(b'header=0x40002004', b'header=0x60002004'))
            for bad in [wrong, raw.replace(b'RAW_RETURN', b'MISSING'),
                        raw.replace(b'pointer=536875000', b'pointer=0'),
                        raw.replace(b'capacity=147628', b'capacity=8196'),
                        raw.replace(expected.encode(), b'other assertion'),
                        raw.replace(b'1 failed', b'0 failed'),
                        raw + b'\nPUBLIC_GC_OOM_CALLER ',
                        raw + b'\nPUBLIC_GC_OOM_BOUND',
                        raw + b'\nPUBLIC_GC_OOM_BOUND source=x',
                        raw + b'\ndid not reach expected fatal terminal within',
                        raw + b'\nerror: could not compile']:
                self.assertFalse(controls.detects(bad, 101, expected))
            for code in [0, 1, 124, -9, None]:
                self.assertFalse(controls.detects(raw, code, expected))
            self.assertFalse(controls.detects(raw, 101, 'unrecognized assertion'))
            self.assertFalse(controls.detects(raw + raw, 101, expected))
            # A valid boundary is not the distinct exhausted-step-bound marker.
            self.assertTrue(controls.detects(raw + f'\nPUBLIC_GC_OOM_BOUNDARY {PREFIX} completed=18'.encode(), 101, expected))

    def test_fatal_mutant_requires_exhaustion_shape(self):
        kind = 'wrong-public-oom-policy'
        raw = manufactured(kind)
        for before, after in [(b'attempt=19', b'attempt=18'), (b'completed=18', b'completed=1'),
                              (b'block_calls=2', b'block_calls=1'), (b'raw_calls=1', b'raw_calls=2'),
                              (b'collections=5', b'collections=0')]:
            self.assertFalse(controls.detects(raw.replace(before, after), 101, controls.MUTANTS[kind][-1]))


if __name__ == '__main__':
    unittest.main()
