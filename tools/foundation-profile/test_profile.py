# SPDX-License-Identifier: MIT
"""Synthetic inventory refusal controls, not independent guest evidence."""
import copy
import ast
import importlib.util
import json
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('profile_checker', Path(__file__).with_name('check_profile.py'))
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)
RECORD = json.loads((ROOT / 'profiles/armv7em-foundations.json').read_text())
BYTES = {name: (ROOT / name).read_bytes() for name in checker.FILES}


class InventoryControls(unittest.TestCase):
    def test_reviewed_inventory(self):
        result = checker.validate(RECORD, BYTES.__getitem__, BYTES.__getitem__, checker.FIXTURES)
        self.assertEqual(result['status'], 'SOURCE_PROFILE_ONLY')
        self.assertFalse(result['runtime_admission'])
        self.assertFalse(result['runtime_build_selection'])

    def test_changed_or_unknown_inputs(self):
        for mutation in ['source', 'base', 'schema-bool', 'admission', 'build', 'missing',
                         'vendor', 'traversal', 'fixture', 'hash', 'extra']:
            with self.subTest(mutation=mutation):
                record = copy.deepcopy(RECORD)
                if mutation == 'source': record['source'] = '0' * 40
                elif mutation == 'base': record['historical_base'] = '0' * 40
                elif mutation == 'schema-bool': record['schema'] = True
                elif mutation == 'admission': record['runtime_admission'] = True
                elif mutation == 'build': record['runtime_build_selection'] = True
                elif mutation == 'missing': del record['files'][next(iter(record['files']))]
                elif mutation == 'vendor': record['files']['asf4/hal.c'] = '0' * 64
                elif mutation == 'traversal': record['files']['../input'] = '0' * 64
                elif mutation == 'fixture': record['fixtures']['primask-state'].append('src/ZSPI.cpp')
                elif mutation == 'hash': record['files'][next(iter(record['files']))] = '0' * 64
                elif mutation == 'extra': record['compiler'] = 'unreviewed'
                with self.assertRaises(ValueError):
                    checker.validate(record, BYTES.__getitem__, BYTES.__getitem__)
        changed = dict(BYTES)
        changed[next(iter(changed))] += b'\n'
        with self.assertRaises(ValueError):
            checker.validate(RECORD, changed.__getitem__, BYTES.__getitem__)

    def test_required_services_fail_closed(self):
        for service in ['polling-spi', 'dma', 'gpio', 'uart', 'adc', 'usb', 'native-arcade']:
            with self.subTest(service=service), self.assertRaises(ValueError):
                checker.validate(RECORD, BYTES.__getitem__, BYTES.__getitem__, [service])

    def test_duplicate_json_keys(self):
        with self.assertRaises(ValueError):
            json.loads('{"schema":1,"schema":1}', object_pairs_hook=checker.closed_object)

    def test_push_and_pr_watch_all_bound_inputs(self):
        workflow = (ROOT / '.github/workflows/simulation-foundation-profile.yml').read_text()
        selectors = re.findall(r'paths: (\[[^\n]+\])', workflow)
        self.assertEqual(len(selectors), 2, 'both push and PR must have explicit selectors')
        expected = checker.FILES | {
            'profiles/armv7em-foundations.json', 'tools/foundation-profile/**',
            '.github/workflows/simulation-foundation-profile.yml'}
        for selector in selectors:
            paths = ast.literal_eval(selector)
            self.assertEqual(len(paths), len(expected), 'no duplicate selectors')
            self.assertEqual(set(paths), expected, 'every bound input must trigger the guard')


if __name__ == '__main__':
    unittest.main()
