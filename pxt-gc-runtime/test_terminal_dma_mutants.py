# SPDX-License-Identifier: MIT
import os
import unittest
from pathlib import Path
from unittest.mock import patch
import check_terminal_dma_mutants as m

class Controls(unittest.TestCase):
    def test_exact_source_mutations_and_preimage_refusal(self):
        for case in m.CASES:
            for kind, (source, label, before, after, expected, marker) in m.MUTANTS.items():
                original = (m.ROOT / source.format(case=case)).read_bytes()
                self.assertEqual(m.mutate(kind, original), original.replace(before, after, 1))
                for bad in [b'', before+b'\n'+before]:
                    with self.assertRaises(ValueError): m.mutate(kind,bad)

    def test_actual_mode_source_image_and_wrong_witness_refusals(self):
        source,image='1'*40,'2'*64
        for case,(outcome,active) in m.CASES.items():
            observations={'wrong-failure-result':f'step=100 actual=0 expected={outcome}',
                          'duplicate-original-failure':f'step=100 pc=0x1000 returned=true active={active}',
                          'omit-native-free':'step=100 actual=0 expected=2',
                          'omit-spi-cleanup-disable':f'step=100 actual=131086 expected=131084 active={active}'}
            for kind,observation in observations.items():
                expected,marker=m.MUTANTS[kind][-2:]
                raw=(f'ALLOCATED_DMA_TERMINAL_IMAGE source={source} elf_sha256={image} case={case}\n'
                     f'{marker} {observation}\n{expected}\n'
                     'test result: FAILED. 0 passed; 1 failed; 0 ignored;\n').encode()
                self.assertTrue(m.detects(raw,101,kind,source,image,case))
                for code in [0,1,None]:self.assertFalse(m.detects(raw,code,kind,source,image,case))
                for before,after in [(marker,'FOREIGN'),(source,'0'*40),(image,'0'*64),(case,'other'),(expected,'unrelated'),('expected='+str(outcome),'expected=99'),('returned=true','returned=false')]:
                    if before.encode() in raw:self.assertFalse(m.detects(raw.replace(before.encode(),after.encode()),101,kind,source,image,case))
                for extra in [b'architectural fault',b'exceeded unchanged bounded budget',b'error: could not compile',f'{marker} {observation}\n'.encode()]:
                    self.assertFalse(m.detects(raw+extra,101,kind,source,image,case))

    def test_hosted_admission_precedes_any_filesystem_work(self):
        with patch.dict(os.environ, {}, clear=True), patch.object(Path, 'exists', side_effect=AssertionError('no work before admission')):
            with self.assertRaises(ValueError): m.main()

if __name__ == '__main__': unittest.main()
