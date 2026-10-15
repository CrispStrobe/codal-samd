# SPDX-License-Identifier: MIT
"""Pure candidate controls, never IRQ/collector execution evidence."""
from pathlib import Path
import re
import unittest
from unittest import mock

import check_routing_mutants as controls


class RoutingControls(unittest.TestCase):
    def test_exact_routing_mutations_and_missing_sites_refuse(self):
        for kind, (_, _, before, after, _) in controls.MUTANTS.items():
            original = b'prefix\n' + before + b'\nsuffix'
            self.assertEqual(controls.mutate(kind, original), original.replace(before, after))
            for bad in [b'', before * 2]:
                with self.assertRaises(ValueError):
                    controls.mutate(kind, bad)

    def test_only_actual_named_assertion_not_other_failures(self):
        for _, _, _, _, expected in controls.MUTANTS.values():
            raw = (expected + '\ntest result: FAILED. 0 passed; 1 failed; 0 ignored;').encode()
            self.assertTrue(controls.detects(raw, 101, expected))
            for code in [0, 1, -9, None]:
                self.assertFalse(controls.detects(raw, code, expected))
            for extra in [b'\nPXT_GC_ROUTING_CALLER ', b'\nPXT_GC_ROUTING_BOUND',
                          b'\ndid not reach terminal within', b'\nerror: could not compile']:
                self.assertFalse(controls.detects(raw + extra, 101, expected))

    def test_no_compiler_before_hosted_admission(self):
        with mock.patch.dict(controls.os.environ, {'GITHUB_ACTIONS': 'false'}), \
             mock.patch.object(controls.subprocess, 'run') as run:
            with self.assertRaisesRegex(ValueError, 'hosted CI only'):
                controls.main()
            run.assert_not_called()

    def test_new_vectors_only_replace_irq0_and_retain_startup(self):
        root = Path(__file__).resolve().parent
        baseline = (root / 'startup.S').read_text()
        candidate = (root / 'routing-startup.S').read_text()
        candidate = candidate.replace(
            'Explicit allocation-routing startup; actual IRQ0 and static constructors.',
            'Separately selected modeled PXT GC startup with actual static constructors.')
        candidate = candidate.replace('.word authored_gc_routing_irq0\n.rept 239', '.rept 240')
        self.assertEqual(candidate, baseline)
        probe = (root / 'probe-routing.cpp').read_text()
        self.assertIn('DEF_VTABLE(routing_vtable, RefRefLocal, ValType::Object,', probe)
        self.assertIn('RefRefLocal::scan(value);', probe)
        self.assertIn('reg(0xe000e200) = 1;', probe)
        self.assertIsNone(re.search(r'\binGC\s*(?:=(?!=)|[|&^+\-]=|\+\+|--)', probe))
        self.assertIsNone(re.search(r'(?:\+\+|--)\s*inGC\b', probe))


if __name__ == '__main__':
    unittest.main()
