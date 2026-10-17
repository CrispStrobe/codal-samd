# SPDX-License-Identifier: MIT
from pathlib import Path
import unittest
from unittest import mock
import check_premature_dma_mutants as m

class Controls(unittest.TestCase):
    def test_unique_original_sites_and_only_exact_copied_source_edits(self):
        self.assertEqual(set(m.MUTANTS),{'omit-native-free','omit-raw-free','disable-dma-before-free'})
        for kind,(path,label,before,after,message,marker) in m.MUTANTS.items():
            original=(m.ROOT/path).read_bytes()
            self.assertEqual(original.count(before),1)
            self.assertEqual(m.mutate(kind,original).replace(after,before,1),original)
            for bad in [b'',before+before]:
                with self.assertRaises(ValueError):m.mutate(kind,bad)

    def test_only_actual_wrong_counts_or_disabled_channel_admitted(self):
        source='1'*40;image='2'*64
        fields={'omit-native-free':'step=99 native=0 raw=0 expected_native=1 expected_raw=1',
                'omit-raw-free':'step=99 native=1 raw=0 expected_native=1 expected_raw=1',
                'disable-dma-before-free':'step=99 channel=2100480 expected_enabled=true'}
        for kind,(_,_,_,_,message,marker) in m.MUTANTS.items():
            lines=[f'ALLOCATED_DMA_PREMATURE_IMAGE source={source} elf_sha256={image} case=allocated-dma-premature-release',f'{marker} {fields[kind]}',message,'test result: FAILED. 0 passed; 1 failed; 0 ignored;']
            raw=('\n'.join(lines)+'\n').encode();self.assertTrue(m.detects(raw,101,kind,source,image))
            for code in [0,1,None]:self.assertFalse(m.detects(raw,code,kind,source,image))
            for line in lines:self.assertFalse(m.detects(raw.replace((line+'\n').encode(),b''),101,kind,source,image))
            for bad in [raw.replace(source.encode(),('0'*40).encode()),raw.replace(image.encode(),('0'*64).encode()),raw.replace(fields[kind].encode(),b'step=99 native=1 raw=1 expected_native=1 expected_raw=1'),raw+f'{marker} {fields[kind]}\n'.encode(),raw+b'ALLOCATED_DMA_PREMATURE_RESULT forbidden\n',raw+b'ALLOCATED_DMA_PREMATURE_WITNESS forbidden\n',raw+b'architectural fault\n',raw+b'error: could not compile\n',raw+b'exceeded unchanged bounded budget\n',b'\xff']:
                self.assertFalse(m.detects(bad,101,kind,source,image))
            if kind=='disable-dma-before-free':self.assertFalse(m.detects(raw.replace(b'channel=2100480',b'channel=2100482'),101,kind,source,image))

    def test_no_work_before_hosted_admission(self):
        with mock.patch.dict(m.os.environ,{'GITHUB_ACTIONS':'false'}),mock.patch.object(m.subprocess,'run') as run,mock.patch.object(m.subprocess,'check_output') as output:
            with self.assertRaisesRegex(ValueError,'hosted CI only'):m.main()
            run.assert_not_called();output.assert_not_called()
