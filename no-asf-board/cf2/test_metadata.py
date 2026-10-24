#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Source/data controls only. Never executes or substitutes for an ARM consumer."""
from pathlib import Path
import copy,importlib.util,json,struct,sys,unittest
spec=importlib.util.spec_from_file_location('metadata',Path(__file__).with_name('metadata.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
HEADER=Path(sys.argv.pop(1)).read_bytes();NOTICE=Path(sys.argv.pop(1)).read_bytes();PROFILE=json.loads(m.PROFILE.read_bytes());WORDS=PROFILE['words']
class Controls(unittest.TestCase):
 def test_exact_actual_source_and_authored_layout(self):
  summary,rows=m.admit(HEADER,NOTICE,PROFILE);self.assertEqual(summary,dict(cf2_bytes=816,used=62,total=100,uf2_information_bytes=16))
  # Separate byte-offset reader for the actual authored data, no producer decoder.
  fragments=dict(rows);self.assertEqual(struct.unpack('<IIII',fragments[0x3ff0]),(0x3000,0,0,0x3400));table=fragments[0x3000]
  self.assertEqual(struct.unpack_from('<IIII',table),(513675505,539130489,62,100))
  pairs={struct.unpack_from('<II',table,16+8*i)[0]:struct.unpack_from('<II',table,16+8*i)[1] for i in range(62)}
  self.assertEqual(len(pairs),62);self.assertEqual(pairs[32],0x2d);self.assertEqual(pairs[204],512*1024);self.assertEqual(pairs[205],192*1024);self.assertEqual(pairs[208],0x239a0033);self.assertEqual(pairs[209],0x55114460)
  self.assertEqual(table[16+62*8:],bytes(38*8));self.assertEqual(fragments[0x3400][-1],0)
 def test_source_and_notice_drift_are_refused(self):
  for header,notice in [(HEADER+b'\n',NOTICE),(HEADER,NOTICE+b'\n'),(HEADER.replace(b'USB_VID 0x239A',b'USB_VID 0x239B'),NOTICE)]:
   self.assertTrue(header!=HEADER or notice!=NOTICE)
   with self.assertRaises(ValueError):m.admit(header,notice,PROFILE)
 def test_profile_cannot_broaden_qualification_or_change_selection(self):
  for field in m.FALSE:
   p=copy.deepcopy(PROFILE);p[field]=True
   with self.assertRaises(ValueError):m.admit(HEADER,NOTICE,p)
  p=copy.deepcopy(PROFILE);p['words'][5]^=1
  with self.assertRaises(ValueError):m.admit(HEADER,NOTICE,p)
 def test_word_mutations_are_nonvacuous_and_specific(self):
  changes=[(0,0),(1,0),(2,0),(2,101),(3,101),(4,0),(6,WORDS[4]),(128,1),(129,1),(5,True),(5,-1),(5,2**32)]
  for index,value in changes:
   changed=list(WORDS);self.assertNotEqual(changed[index],value);changed[index]=value
   with self.subTest(index=index,value=value):
    with self.assertRaises(ValueError):m.validate_words(changed)
  for changed in [WORDS[:-1],WORDS+[0],WORDS[:4]]:
   with self.assertRaises(ValueError):m.validate_words(changed)
 def test_only_literal_array_is_selected(self):
  for before,after in [(b'513675505',b'(513675505)'),(b'513675505',b'513675505+0'),(b'/* CF2 END */',b'/* CF2 END */ /* CF2 END */'),(b'/* CF2 START */',b'/* CF2 START */ /* CF2 START */')]:
   self.assertIn(before,HEADER)
   with self.assertRaises(ValueError):m.words(HEADER.replace(before,after))
 def test_fragment_admission_refuses_overlap_alignment_handover_and_strings(self):
  original=m.segments(WORDS)
  mutated=[[(0,b'bad')]+original[1:],[original[0],(0x3004,original[1][1]),original[2]],original[:2]+[(0x3ff0,struct.pack('<IIII',0x3002,0,0,0x3400))],original[:2]+[(0x3ff0,struct.pack('<IIII',0x3000,0x101,0,0x3400))],original[:2]+[(0x3ff0,struct.pack('<IIII',0x3000,0,0x101,0x3400))],original[:2]+[(0x3ff0,struct.pack('<IIII',0x3000,0,0,0x4000))],[original[0],(0x3400,original[1][1][:-1]),original[2]],[original[0],(0x3400,b'\0'+original[1][1]),original[2]],[original[0],(0x3400,b'X'*129+b'\0'),original[2]],original[:2]+[(0x3ff0,original[2][1]+b'\0')]]
  for rows in mutated:
   self.assertNotEqual(rows,original)
   with self.assertRaises(ValueError):m.validate_segments(rows,WORDS)
if __name__=='__main__':unittest.main()
