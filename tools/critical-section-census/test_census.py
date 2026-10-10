# SPDX-License-Identifier: MIT
"""Synthetic refusal controls, not a replay of the actual dependency capture."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile

import census


def synthetic_report():
    files = {}
    units = []
    for path, digest in census.FILES.items():
        source = 'libraries/' + path
        dep = 'build-source/' + source
        files[dep] = {'sha256': digest, 'path': source, 'origin': 'build-source'}
        if path.endswith('.cpp'):
            units.append({'source': source, 'sourceSha256': digest, 'dependencies': [dep]})
    while len(units) < 172:
        units.append({'source': f'pxtapp/synthetic/probe{len(units)}.cpp', 'dependencies': []})
    files['build-source/build/codal_extra_definitions.h'] = {'sha256': census.CONFIG_SHA256, 'bytes': 1634}
    return {'schema': 1, 'completeSourceToBinaryProof': False, 'units': units, 'files': files}


class CensusControls(unittest.TestCase):
    def test_synthetic_positive_boundary(self):
        out = census.summarize(synthetic_report())
        self.assertEqual(out['status'], 'SOURCE_CENSUS_ONLY')
        self.assertEqual(len(out['files']), 15)
        for flag in ('runtime_admission', 'runtime_build_selection', 'active_call_proof'):
            self.assertIs(out[flag], False)
        self.assertEqual(len(out['unresolved']), 6)
        self.assertEqual(out['pins'], census.PINS)

    def test_record_mutations_refuse(self):
        mutations = [
            lambda r: r.update(schema=True),
            lambda r: r.update(completeSourceToBinaryProof=True),
            lambda r: r['units'].pop(),
            lambda r: r['units'].__setitem__(1, copy.deepcopy(r['units'][0])),
            lambda r: r['units'][0].update(sourceSha256='0' * 64),
            lambda r: r['units'][0].update(dependencies='not-a-list'),
            lambda r: r['units'][0].update(source='/private/source.cpp'),
            lambda r: r['units'][0].update(source='libraries/../private.cpp'),
            lambda r: r['units'][0].update(source='pxtapp/hidden\npath.cpp'),
            lambda r: r['files'].pop('build-source/libraries/codal-core/inc/core/codal_target_hal.h'),
            lambda r: r['files']['build-source/libraries/codal-core/inc/core/codal_target_hal.h'].update(origin='unbound'),
            lambda r: r['files']['build-source/build/codal_extra_definitions.h'].update(bytes=True),
            lambda r: r['files']['build-source/build/codal_extra_definitions.h'].update(sha256='0' * 64),
        ]
        for i, mutate in enumerate(mutations):
            with self.subTest(mutation=i):
                report = synthetic_report()
                mutate(report)
                with self.assertRaises(ValueError):
                    census.summarize(report)

    def test_exact_byte_and_duplicate_json_controls(self):
        data = b'synthetic control only'
        census.bind_bytes(data, hashlib.sha256(data).hexdigest())
        with self.assertRaises(ValueError):
            census.bind_bytes(data + b'!', hashlib.sha256(data).hexdigest())
        with self.assertRaises(ValueError):
            json.loads('{"schema":1,"schema":1}', object_pairs_hook=census.closed_object)

    def test_archive_refuses_missing_and_wrong_original(self):
        # Small owned synthetic files; no actual capture is downloaded or replayed.
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'synthetic.zip'
            with zipfile.ZipFile(path, 'w') as archive:
                archive.writestr('unrelated.txt', b'control')
            with self.assertRaises(ValueError):
                census.read_archive(path)
            result = subprocess.run(
                [sys.executable, '-B', str(Path(__file__).with_name('census.py')), str(path)],
                capture_output=True, text=True, check=False)
            self.assertEqual(result.returncode, 2)
            self.assertEqual(result.stdout, '')
            self.assertEqual(result.stderr, 'source census refused: missing, malformed or mismatched retained evidence\n')
            with zipfile.ZipFile(path, 'w') as archive:
                archive.writestr('diagnostic-dependencies.json', json.dumps(synthetic_report()))
                archive.writestr('generated/build/codal_extra_definitions.h', b'not the original')
            with self.assertRaises(ValueError):
                census.read_archive(path)


if __name__ == '__main__':
    unittest.main()
