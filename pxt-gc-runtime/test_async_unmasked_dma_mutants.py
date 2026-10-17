# SPDX-License-Identifier: MIT
from pathlib import Path
import unittest
from unittest import mock
import check_async_unmasked_dma_mutants as m


class AsyncUnmaskedDMAMutantControls(unittest.TestCase):
    def test_declared_sources_have_one_exact_site_and_only_that_edit(self):
        self.assertEqual(set(m.MUTANTS), {'wrong-native-byte', 'duplicate-original-callback',
                                         'omit-native-free', 'omit-pending-collections',
                                         'omit-quiet-loop', 'mask-quiet-phase'})
        for kind, (source, _, before, after, _, _) in m.MUTANTS.items():
            original = (m.ROOT / source).read_bytes()
            self.assertEqual(original.count(before), 1)
            changed = m.mutate(kind, original)
            self.assertEqual(changed.replace(after, before, 1), original)
            for invalid in [b'', before + before]:
                with self.assertRaises(ValueError):
                    m.mutate(kind, invalid)

    def test_only_actual_named_failure_shape_and_exact_image_are_admitted(self):
        source, image = '1' * 40, '2' * 64
        values = {'wrong-native-byte': 'pointer=536875008 index=7 actual=15 expected=14',
                  'duplicate-original-callback': 'step=99 pc=0x1234 returned=true active=79',
                  'omit-native-free': 'step=99 actual=0 expected=2',
                  'omit-pending-collections': 'step=99 actual=0 expected=2',
                  'omit-quiet-loop': 'step=99 actual=9 expected_min=2048',
                  'mask-quiet-phase': 'step=99 active=0 primask=true reason=masked-thread'}
        for kind, (_, _, _, _, expected, marker) in m.MUTANTS.items():
            raw = (f'ALLOCATED_DMA_ASYNC_UNMASKED_IMAGE source={source} elf_sha256={image} case=allocated-dma-async-unmasked\n'
                   f'{marker} {values[kind]}\n{expected}\n'
                   'test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out;\n').encode()
            self.assertTrue(m.detects(raw, 101, kind, source, image))
            for code in [0, 1, None]:
                self.assertFalse(m.detects(raw, code, kind, source, image))
            self.assertFalse(m.detects(raw, 101, kind, '0' * 40, image))
            self.assertFalse(m.detects(raw, 101, kind, source, '0' * 64))
            for line in raw.splitlines(keepends=True):
                self.assertFalse(m.detects(raw.replace(line, b''), 101, kind, source, image))
            for invalid in [raw + b'ALLOCATED_DMA_ASYNC_UNMASKED_RESULT forbidden\n',
                            raw + b'exceeded unchanged bounded budget\n',
                            raw + b'architectural fault\n', raw + b'error: could not compile\n',
                            raw + (marker + ' ' + values[kind] + '\n').encode(), b'\xff']:
                self.assertFalse(m.detects(invalid, 101, kind, source, image))

    def test_wrong_payload_pointer_and_wrong_terminal_context_refuse(self):
        source, image = '1' * 40, '2' * 64
        for kind, bad in [('wrong-native-byte', 'pointer=536875009 index=7 actual=15 expected=14'),
                          ('wrong-native-byte', 'pointer=536875008 index=7 actual=14 expected=14'),
                          ('duplicate-original-callback', 'step=99 pc=0x1234 returned=false active=79'),
                          ('omit-native-free', 'step=99 actual=2 expected=2'),
                          ('omit-quiet-loop', 'step=99 actual=2048 expected_min=2048'),
                          ('mask-quiet-phase', 'step=99 active=0 primask=false reason=masked-thread')]:
            expected, marker = m.MUTANTS[kind][-2:]
            raw = (f'ALLOCATED_DMA_ASYNC_UNMASKED_IMAGE source={source} elf_sha256={image} case=allocated-dma-async-unmasked\n'
                   f'{marker} {bad}\n{expected}\ntest result: FAILED. 0 passed; 1 failed; 0 ignored;\n').encode()
            self.assertFalse(m.detects(raw, 101, kind, source, image))

    def test_no_build_or_guest_work_before_hosted_admission(self):
        with mock.patch.dict(m.os.environ, {'GITHUB_ACTIONS': 'false'}), \
             mock.patch.object(m.subprocess, 'run') as run, \
             mock.patch.object(m.subprocess, 'check_output') as output:
            with self.assertRaisesRegex(ValueError, 'hosted CI only'):
                m.main()
            run.assert_not_called()
            output.assert_not_called()
