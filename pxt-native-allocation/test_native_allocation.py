# SPDX-License-Identifier: MIT
"""Source-preservation controls, not allocator execution or GC qualification."""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]


def expected_native(service):
    service = service.replace(
        '// Simulation-only public allocation domain over the retained actual CODAL heap.\n'
        '// Raw device_* pointers and public aligned pointers must not be interchanged.',
        '// Simulation-only explicitly named native allocation domain for PXT coexistence.\n'
        '// PXT owns malloc/free/realloc; these pointers must use only bw_native_* APIs.')
    service = service.replace('#include "platform_includes.h"', '#include "native-aligned-heap.h"')
    for old, new in [('malloc', 'bw_native_alloc'), ('free', 'bw_native_free'),
                     ('realloc', 'bw_native_realloc')]:
        service = re.sub(r'\b' + old + r'\b', new, service)
    return service.replace('// PXT owns bw_native_alloc/bw_native_free/bw_native_realloc;',
                           '// PXT owns malloc/free/realloc;')


class NativeAllocationControls(unittest.TestCase):
    def test_exact_existing_algorithm_with_distinct_names(self):
        service = (ROOT / 'runtime-services/aligned-heap.cpp').read_text()
        native = (ROOT / 'pxt-native-allocation/native-aligned-heap.cpp').read_text()
        self.assertEqual(native, expected_native(service))

    def test_no_public_allocator_definition(self):
        native = (ROOT / 'pxt-native-allocation/native-aligned-heap.cpp').read_text()
        self.assertEqual(set(re.findall(r'extern "C" void\s*\*?\s*(\w+)\(', native)),
                         {'bw_native_alloc', 'bw_native_free', 'bw_native_realloc'})

    def test_existing_service_selection_unchanged(self):
        build = (ROOT / 'runtime-services/build.sh').read_text()
        self.assertNotIn('pxt-native-allocation', build)
        self.assertIn('own_sources=(clock target probe aligned-heap)', build)

    def test_explicit_separate_selector_and_domain_contract(self):
        header = (ROOT / 'pxt-native-allocation/native-aligned-heap.h').read_text()
        self.assertIn('#if !defined(BW_PXT_NATIVE_ALLOCATION)', header)
        self.assertIn('#error "Select the distinct PXT native allocation domain explicitly"', header)
        self.assertIn('#include "platform_includes.h"', header)
        self.assertIn('are not GC roots', header)
        self.assertIn('must not be\n// passed to public free/realloc', header)


if __name__ == '__main__':
    unittest.main()
