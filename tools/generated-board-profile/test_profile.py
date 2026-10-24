# SPDX-License-Identifier: MIT
import importlib.util, json, os, tempfile, unittest
from pathlib import Path
from unittest.mock import patch
spec=importlib.util.spec_from_file_location('generated_profile',Path(__file__).with_name('verify.py'))
v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
class Controls(unittest.TestCase):
    def test_real_source_reconstruction_and_changed_inputs(self):
        source=Path(os.environ['BOARD_CODAL_SOURCE']);board=Path(os.environ['BOARD_TARGET_SOURCE'])
        with tempfile.TemporaryDirectory(dir=v.ROOT) as folder:
            temporary=Path(folder)
            v.verify(source,board,temporary)
            profile=json.loads((v.ROOT/'profiles/pxt-generated-board-profile.json').read_bytes())
            for item in profile['public_inputs']:
                path=({'codal':source,'codal-itsybitsy-m4':board}[item['repo']]/item['path'])
                original=path.read_bytes()
                try:
                    path.write_bytes(original+b'\n')
                    with self.subTest(path=item['path']),self.assertRaisesRegex(ValueError,'public input identity'):
                        v.verify(source,board,temporary)
                finally:path.write_bytes(original)
            app=v.ROOT/'tools/generated-board-profile/empty-project-codal.json';original=app.read_bytes()
            try:
                app.write_bytes(original.replace(b'"DEVICE_USB": 1',b'"DEVICE_USB": 0'))
                with self.assertRaisesRegex(ValueError,'application identity'):v.verify(source,board,temporary)
            finally:app.write_bytes(original)
            v.verify(source,board,temporary)
if __name__=='__main__':unittest.main()
