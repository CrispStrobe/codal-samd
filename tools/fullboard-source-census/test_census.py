# SPDX-License-Identifier: MIT
import json
import unittest
import census

class Controls(unittest.TestCase):
 def test_missing_each_source_witness_refuses(self):
  for paths in [census.OWN,census.PUBLIC]:
   for p,tokens in paths.items():
    raw=('\n'.join(tokens)+'\n').encode()
    census.witness(raw,tokens)
    for token in tokens:
     with self.subTest(path=p,token=token),self.assertRaises(ValueError):
      census.witness(raw.replace(token.encode(),b''),tokens)
 def test_config_is_literal_and_unique(self):
  for n in ['DEVICE_USB','DEVICE_WEBUSB']:
   for v in [0,1]:self.assertEqual(census.config(('#define '+n+' '+str(v)+'\n').encode(),n),v)
   for raw in [b'',('#define '+n+' CONFIG\n').encode(),('#define '+n+' 1\n#define '+n+' 0\n').encode()]:
    with self.assertRaises(ValueError):census.config(raw,n)
 def test_source_use_absence_changes_when_present(self):
  self.assertEqual(census.projection_uses({'a':b'no use'},'QSPI'),{})
  self.assertEqual(census.projection_uses({'a':b'QSPI->CTRLA\n'},'QSPI'),{'a':[1]})
 def test_committed_report_boundaries(self):
  r=json.loads((census.ROOT/'profiles/pxt-fullboard-source-census.json').read_bytes())
  self.assertEqual(r['engine']['pin'],census.ENGINE)
  self.assertEqual(r['codal']['commit'],census.CODAL)
  self.assertEqual(r['core']['commit'],census.CORE)
  self.assertEqual(r['projected_public_files'],64)
  self.assertEqual(r['configuration']['core_defaults'],dict(DEVICE_USB=0,DEVICE_WEBUSB=1))
  self.assertEqual(r['configuration']['qualified_service'],dict(DEVICE_USB=0,DEVICE_WEBUSB=0))
  self.assertEqual(r['qspi']['projected_source_uses'],{})
  self.assertEqual([x['id'] for x in r['engine']['descriptor']['required_addresses_not_declared']],['nvmctrl','cmcc'])
  self.assertEqual({x['id']:x['type'] for x in r['engine']['descriptor']['entries'] if x['id'] in ['usb','qspi']},dict(usb='stub',qspi='stub'))
  self.assertEqual(len(r['engine']['native_model_sources']),2)
  for k in ['compiled','linked','executed','fullboard','usb_qualified','qspi_required','licence_clearance']:self.assertIs(r[k],False)
if __name__=='__main__':unittest.main()
