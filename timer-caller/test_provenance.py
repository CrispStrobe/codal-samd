#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Manufactured reader controls only; no compiler or guest execution."""
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('timer_provenance', Path(__file__).with_name('provenance.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class DependencyControls(unittest.TestCase):
    def test_complete_rule_and_escaped_space(self):
        self.assertEqual(module.dependencies('unit.o: source.cpp \\\n /headers/with\\ space.h\n'),
                         ['source.cpp', '/headers/with space.h'])

    def test_missing_or_multiple_rules_refuse(self):
        for text in ('', ': input.h', 'unit.o:', 'unit.o: a.h\nother.o: b.h'):
            with self.assertRaises(ValueError):
                module.dependencies(text)

    def test_repeated_headers_preserved_but_extra_target_refuses(self):
        self.assertEqual(module.dependencies('unit.o: a.h a.h'), ['a.h', 'a.h'])
        for text in ('unit.o: a.h other.o: b.h',):
            with self.assertRaises(ValueError):
                module.dependencies(text)

    def test_data_binding_is_content_specific(self):
        self.assertEqual(module.record(b'abc')['bytes'], 3)
        self.assertNotEqual(module.record(b'abc'), module.record(b'abd'))

    def verification_fixture(self):
        labels = ['core/' + name for name in module.CORE if name != 'LICENSE']
        labels += ['samd/paired-irq/profile.cpp', 'samd/timer-caller/probe.cpp']
        paths = {Path('/manufactured/' + label).resolve(): label for label in labels}
        manifest = {'files': {label: module.record(b'original') for label in labels}}
        units = {'timer': [label for label in labels if label.startswith('core/')],
                 'profile': ['samd/paired-irq/profile.cpp'], 'probe': ['samd/timer-caller/probe.cpp']}
        return paths, manifest, units

    def verify_manufactured(self, paths, manifest, units, changed=None):
        def text(path):
            name = path.stem
            return 'object.o: ' + ' '.join('/manufactured/' + label for label in units[name])
        def data(path):
            if changed and path == Path('/manufactured/' + changed):
                return b'changed'
            return b'original'
        with patch.object(Path, 'read_text', text), patch.object(Path, 'read_bytes', data):
            return module.verify(Path('/manufactured/output'), manifest, paths)

    def test_bound_complete_closure_accepts(self):
        paths, manifest, units = self.verification_fixture()
        units['timer'].extend(['core/inc/core/CodalConfig.h', 'core/inc/core/ErrorNo.h'])
        result = self.verify_manufactured(paths, manifest, units)
        self.assertEqual(len(result['units']['timer']), len(set(result['units']['timer'])))
        self.assertFalse(result['guest_executed'])
        self.assertFalse(result['runtime_admitted'])

    def test_unknown_drift_missing_header_and_wrong_source_refuse(self):
        for adversary in ('unknown', 'drift', 'missing', 'wrong_source'):
            paths, manifest, units = self.verification_fixture()
            changed = None
            if adversary == 'unknown':
                units['timer'].extend(['unbound/injected.h', 'unbound/injected.h'])
            elif adversary == 'drift':
                changed = units['timer'][0]
            elif adversary == 'missing':
                units['timer'].remove('core/inc/types/Event.h')
            else:
                units['probe'] = ['samd/paired-irq/profile.cpp']
            with self.subTest(adversary=adversary), self.assertRaises(ValueError):
                self.verify_manufactured(paths, manifest, units, changed)


if __name__ == '__main__':
    unittest.main()
