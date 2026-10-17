# SPDX-License-Identifier: MIT
import os
import unittest
from pathlib import Path
from unittest.mock import patch
import check_blocking_dma_mutants as m

class Controls(unittest.TestCase):
    def test_exact_source_mutations_and_preimage_refusal(self):
        for kind, (source, label, before, after, expected, marker) in m.MUTANTS.items():
            original = (m.ROOT / source).read_bytes()
            changed = m.mutate(kind, original)
            self.assertEqual(changed, original.replace(before, after, 1))
            self.assertNotEqual(changed, original)
            for bad in [b'', before + b'\n' + before]:
                with self.assertRaises(ValueError): m.mutate(kind, bad)

    def test_actual_source_bound_wrong_witness_and_refusals(self):
        source, image = '1' * 40, '2' * 64
        observations = {'omit-blocking-token-take':'step=100 entries=0 returns=0 expected=1',
            'duplicate-blocking-token-take':'step=100 entries=2 returns=2 result=0 terminal=0 expected=1',
            'wrong-blocking-return':'step=100 actual=1 expected=0'}
        for kind, observation in observations.items():
            expected, marker = m.MUTANTS[kind][-2:]
            raw = (f'ALLOCATED_DMA_BLOCKING_IMAGE source={source} elf_sha256={image} case=allocated-dma-blocking\n'
                   f'{marker} {observation}\n{expected}\n'
                   'test result: FAILED. 0 passed; 1 failed; 0 ignored;\n').encode()
            self.assertTrue(m.detects(raw, 101, kind, source, image))
            for code in [0, 1, None]: self.assertFalse(m.detects(raw, code, kind, source, image))
            for before, after in [(marker,'WRONG'),(observation,observation.replace('=1','=2')),
                                  (source,'0'*40),(image,'0'*64),(expected,'unrelated failure')]:
                self.assertFalse(m.detects(raw.replace(before.encode(),after.encode()),101,kind,source,image))
            for extra in [b'architectural fault',b'exceeded unchanged bounded budget',
                          b'error: could not compile',f'{marker} {observation}\n'.encode()]:
                self.assertFalse(m.detects(raw+extra,101,kind,source,image))

    def test_hosted_admission_precedes_any_filesystem_work(self):
        with patch.dict(os.environ, {}, clear=True), patch.object(Path, 'exists', side_effect=AssertionError('no work before admission')):
            with self.assertRaises(ValueError): m.main()

if __name__ == '__main__': unittest.main()
