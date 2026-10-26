#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Manufactured UF2/metadata admission controls; no guest is executed."""
import importlib.util,json,struct,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('boot',Path(__file__).with_name('boot.py'));b=importlib.util.module_from_spec(s);s.loader.exec_module(b)
def fixture():
 r=bytearray(512);struct.pack_into('<8I',r,0,0x0a324655,0x9e5d5157,0x2000,0x4000,8,0,1,0x55114460);struct.pack_into('<II',r,32,0x20030000,0x4101);struct.pack_into('<I',r,508,0x0ab16f30);return bytes(r)
class Controls(unittest.TestCase):
 def test_default_retains_original_fragments(self):
  image,report=b.assemble(fixture());profile=json.loads((b.ROOT/'profiles/pxt-cf2-authored-metadata.json').read_bytes());self.assertEqual(report['cf2_fragments'],profile['segments']);self.assertIsNone(report['authored_configuration_override']);self.assertEqual(image[0x3000:0x3330],struct.pack('<204I',*profile['words']))
 def test_only_declared_extra_timer_pair_changes(self):
  old,_=b.assemble(fixture());new,report=b.assemble(fixture(),0x20111200);words=list(struct.unpack('<204I',new[0x3000:0x3330]));original=json.loads((b.ROOT/'profiles/pxt-cf2-authored-metadata.json').read_bytes())['words'];self.assertEqual(words[2],63);self.assertEqual(words[128:130],[218,0x20111200]);words[2]=62;words[128:130]=[0,0];self.assertEqual(words,original);self.assertEqual(old[:0x3000],new[:0x3000]);self.assertEqual(old[0x3330:],new[0x3330:]);self.assertEqual(report['authored_configuration_override']['value'],0x20111200)
 def test_undeclared_timer_lists_refuse(self):
  for value in [0,0x10111200,0x20101200,0x20212200]:
   with self.subTest(value=value),self.assertRaises(ValueError):b.assemble(fixture(),value)
if __name__=='__main__':unittest.main()
