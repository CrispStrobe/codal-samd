# SPDX-License-Identifier: MIT
import copy
import json
from pathlib import Path
import unittest

import inventory as reader


class InventoryTests(unittest.TestCase):
    def sample(self):
        source = 'build-source/app.cpp'
        header = 'build-source/header.h'
        files = {name: {'sha256': str(i) * 64}
                 for i, name in enumerate([source, header], 1)}
        origins = {'records': {name: {'kind': 'original-request',
                                      'requestKey': '/' + name.removeprefix('build-source/'),
                                      'sha256': record['sha256']}
                               for name, record in files.items()}}
        diagnostic = {'files': files, 'units': [
            {'source': 'app.cpp', 'sourceSha256': '1' * 64, 'target': 'app.cpp.o',
             'dependencies': [source, header, header],
             'invocation': ['private-host-details-not-to-be-projected']} ]}
        return diagnostic, origins

    def test_full_graph_and_repeated_gcc_prerequisites(self):
        report = reader.graph(*self.sample())
        self.assertEqual(report['repeated_dependency_occurrences'], 1)
        self.assertEqual(report['units'][0]['dependency_file_ids'], [0, 1])
        self.assertNotIn('invocation', json.dumps(report))
        self.assertNotIn('private-host', json.dumps(report))

    def test_graph_refusals(self):
        mutations = [
            lambda d, o: o['records'].pop('build-source/header.h'),
            lambda d, o: d['units'].append(copy.deepcopy(d['units'][0])),
            lambda d, o: d['units'][0].update(sourceSha256='3' * 64),
            lambda d, o: d['units'][0].update(source='../app.cpp'),
            lambda d, o: d['units'][0].update(target='/private/app.o'),
            lambda d, o: d['units'][0].update(dependencies=['build-source/header.h']),
            lambda d, o: d['units'][0]['dependencies'].append('unknown.h'),
            lambda d, o: o['records']['build-source/header.h'].update(sha256='3' * 64),
            lambda d, o: o['records']['build-source/header.h'].update(kind='unknown'),
            lambda d, o: o['records']['build-source/header.h'].update(requestKey='/wrong'),
        ]
        for mutation in mutations:
            with self.subTest(mutation=mutations.index(mutation)):
                d, o = self.sample()
                mutation(d, o)
                with self.assertRaises(ValueError):
                    reader.graph(d, o)

    def test_paths_and_hashes(self):
        for name in ['/absolute', '../escape', 'a/../escape', 'a\\escape', './name', 'a//b', '']:
            with self.subTest(path=name), self.assertRaises(ValueError):
                reader.safe(name)
        for value in [True, 'a' * 63, 'g' * 64, 'A' * 64]:
            with self.subTest(hash=value), self.assertRaises(ValueError):
                reader.sha(value)
        self.assertEqual(reader.safe('relative/path.h'), 'relative/path.h')

    def test_duplicate_json(self):
        with self.assertRaises(ValueError):
            json.loads('{"a":1,"a":2}', object_pairs_hook=reader.closed)

    def map(self):
        return ('Discarded input sections\n'
                ' .text.discarded 0x0 0x20 CMakeFiles/T.dir/pxtapp/settings/SAMDFlash.cpp.o\n'
                'Linker script and memory map\n'
                'LOAD CMakeFiles/T.dir/pxtapp/settings/SAMDFlash.cpp.o\n'
                ' .text.real\n'
                '  0x00002000 0x10 CMakeFiles/T.dir/pxtapp/settings/SAMDFlash.cpp.o\n'
                ' .debug_info 0x00002010 0x80 CMakeFiles/T.dir/pxtapp/settings/SAMDFlash.cpp.o\n'
                ' .text.other 0x00002090 0x10 unrelated.cpp.o\n')

    def test_allocated_text_not_load_discard_or_debug(self):
        selected = reader.storage_sections(self.map())
        self.assertEqual(len(selected), 1)
        self.assertEqual(selected[0]['section'], '.text.real')
        self.assertEqual(selected[0]['address'], 0x2000)

    def test_map_refusals(self):
        for text in [self.map().replace('Linker script and memory map', ''),
                     self.map() + 'Linker script and memory map',
                     self.map().replace('0x00002000', '0x00000000'),
                     self.map().replace('0x00002000', '0x20000000'),
                     self.map().replace('0x10 CMakeFiles', '0x0 CMakeFiles'),
                     self.map() + self.map().split('Linker script and memory map')[1]]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                reader.storage_sections(text)

    def test_committed_report_boundary(self):
        path = Path(__file__).resolve().parents[2] / 'profiles/historical-board-consumers.json'
        report = json.loads(path.read_text(), object_pairs_hook=reader.closed)
        self.assertEqual(len(report['units']), 172)
        self.assertEqual(len(report['files']), 542)
        self.assertEqual(report['archive_sha256'], reader.ARCHIVE_SHA)
        self.assertEqual(report['map_sha256'], reader.MAP_SHA)
        for flag in ['assembly_coverage', 'current_profile_compiled', 'image_admitted',
                     'runtime_qualified', 'licence_clearance']:
            self.assertIs(report[flag], False)
        for unit in report['units']:
            ids = unit['dependency_file_ids']
            self.assertEqual(ids, sorted(set(ids)))
            self.assertTrue(all(type(i) is int and 0 <= i < 542 for i in ids))
        self.assertEqual(len(report['allocated_settings_text']), 28)
        text = path.read_text()
        self.assertNotIn('invocation', text.replace('host invocations', ''))
        self.assertNotRegex(text, r'/(?:home|mnt|tmp|Users)/')


if __name__ == '__main__':
    unittest.main()
