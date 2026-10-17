# SPDX-License-Identifier: MIT
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import census


class SourceCensus(unittest.TestCase):
    def test_all_source_witnesses_are_required(self):
        payloads = {path: ('\n'.join(items.values()) + '\n').encode()
                    for path, items in census.WITNESSES.items()}
        result = census.source_observations(payloads)
        self.assertEqual(set(result), set(census.WITNESSES))
        for path, items in census.WITNESSES.items():
            for token in items.values():
                bad = copy.deepcopy(payloads)
                bad[path] = bad[path].replace(token.encode(), b'')
                with self.subTest(path=path, token=token), self.assertRaises(ValueError):
                    census.source_observations(bad)

    def test_descriptor_records_presence_without_claiming_runtime(self):
        raw = b'flash:\n  size: "512KB"\nperipherals:\n  - id: "usb"\n    type: "stub"\n    base_address: 0x41000000\n'
        result = census.descriptor_observations(raw)
        self.assertEqual([r['id'] for r in result['required_addresses_not_declared']],
                         ['nvmctrl', 'cmcc'])
        # A real declaration at that address changes the census even if named
        # differently; the scanner cannot keep claiming the address is absent.
        extended = raw + b'  - id: "flashctl"\n    type: "stub"\n    base_address: 0x41004000\n'
        result = census.descriptor_observations(extended)
        self.assertEqual([r['id'] for r in result['required_addresses_not_declared']], ['cmcc'])
        self.assertEqual(result['entries'][-1]['type'], 'stub')
        for bad in [raw.replace(b'    type:', b'    unexpected:'),
                    raw + raw.split(b'peripherals:\n')[1],
                    raw.replace(b'flash:', b'include: "base.yaml"\nflash:'),
                    raw.replace(b'peripherals:', b'other:')]:
            with self.subTest(raw=bad), self.assertRaises(ValueError):
                census.descriptor_observations(bad)
        # Comments containing an address are not declarations.
        self.assertEqual(census.descriptor_observations(raw + b'# nvmctrl 0x41004000\n'),
                         census.descriptor_observations(raw))

    def test_original_stage_bytes_and_path_containment(self):
        with tempfile.TemporaryDirectory(dir=Path(__file__).parent) as directory:
            stage = Path(directory)
            (stage / 'LICENSE-pxt-common-packages').write_bytes(b'notice')
            path = stage / 'pxtapp/input.cpp'; path.parent.mkdir(); path.write_bytes(b'actual')
            blob, digest, size = census.sources.identity(b'actual')
            profile = dict(sources=[dict(request='/pxtapp/input.cpp', git_blob=blob,
                                         sha256=digest, bytes=size)])
            with patch.object(census.sources, 'validate'), patch.object(
                    census.sources, 'NOTICE', census.sources.identity(b'notice')[0]):
                self.assertEqual(census.public_payloads(stage, profile),
                                 {'pxtapp/input.cpp': b'actual'})
                path.write_bytes(b'changed')
                with self.assertRaises(ValueError): census.public_payloads(stage, profile)
                path.unlink(); path.symlink_to(stage / 'LICENSE-pxt-common-packages')
                with self.assertRaises(ValueError): census.public_payloads(stage, profile)
                (stage / 'LICENSE-pxt-common-packages').write_bytes(b'changed notice')
                with self.assertRaises(ValueError): census.public_payloads(stage, profile)

    def test_committed_report_has_only_source_claims(self):
        report = json.loads((census.ROOT / 'profiles/pxt-storage-link-census.json').read_bytes())
        self.assertEqual(report['status'], 'PINNED_STORAGE_SOURCE_GAP_CENSUS_ONLY')
        for key in ['compiled', 'linked', 'executed', 'storage_qualified',
                    'original_board_configuration_qualified', 'image_admitted', 'licence_clearance']:
            self.assertIs(report[key], False)
        self.assertEqual(report['public_source_files'], 64)
        self.assertEqual(report['engine']['pin'], census.ENGINE)
        self.assertEqual([r['id'] for r in report['engine']['descriptor']
                          ['required_addresses_not_declared']], ['nvmctrl', 'cmcc'])


if __name__ == '__main__':
    unittest.main()
