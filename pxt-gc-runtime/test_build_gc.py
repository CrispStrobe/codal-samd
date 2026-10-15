# SPDX-License-Identifier: MIT
from pathlib import Path
import unittest
from unittest import mock

import build_gc as gc


def payloads():
    base = b'// actual class bodies are not synthesized by the adapter\n'
    for include, count in gc.BASE_EDITS:
        base += include * count
    return {'pxtapp/pxtbase.h': base,
            'pxtapp/base/gc.cpp': b'#include "pxtbase.h"\n' + gc.REALLOC_COPY + b'\nvoid real_gc_body();\n',
            'pxtapp/base/pxt.cpp': b'#include "pxtbase.h"\nusing namespace std;\nvoid real_object_body();\n',
            'pxtapp/core---samd/codal.cpp': b'ignored prefix\n#ifdef DEVICE_GET_FIBER_LIST_AVAILABLE\n'
                b'int actual_listing();\n#endif\n\nnamespace pxt {\n'
                b'ThreadContext *getThreadContext() {\nreturn actual_context();\n}\n'
                b'void gcProcessStacks(int flags) { real_scan(flags); }\n'
                b'LowLevelTimer *getJACDACTimer() {\nignored suffix',
            'pxtapp/base/core.cpp': b'ignored prefix\nint *getBootloaderConfigData() __attribute__((weak));\n'
                b'int *getBootloaderConfigData() { return NULL; }\n'
                b'int getConfig(int key, int defl) { return real_lookup(key, defl); }\n'
                b'\n} // namespace pxt\n\nnamespace pxtrt {\nignored suffix'}


class GCLinkControls(unittest.TestCase):
    def test_unique_replacements_refuse_missing_or_repeated(self):
        self.assertEqual(gc.once(b'a\nx\nb', b'x\n', b'y\n'), b'a\ny\nb')
        for raw in [b'', b'x\nx\n']:
            with self.assertRaises(ValueError):
                gc.once(raw, b'x\n', b'y\n')

    def test_extraction_refuses_ambiguous_or_reversed_bounds(self):
        self.assertEqual(gc.between(b'xSTARTbodyENDy', b'START', b'END'), b'STARTbody')
        for raw in [b'', b'ENDSTART', b'STARTSTARTEND', b'STARTENDEND']:
            with self.assertRaises(ValueError):
                gc.between(raw, b'START', b'END')

    def test_actual_inputs_adaptation_preserves_functions(self):
        selected = gc.adapt(payloads(), (gc.ROOT / 'runtime-services/target.cpp').read_bytes())
        self.assertTrue(selected['gc-storage.cpp'].endswith(b'void real_gc_body();\n'))
        self.assertTrue(selected['pxt-object-storage.cpp'].endswith(b'void real_object_body();\n'))
        self.assertIn(b'void gcProcessStacks(int flags) { real_scan(flags); }', selected['pxt-fiber-roots.cpp'])
        self.assertIn(b'int actual_listing();', selected['pxt-fiber-roots.cpp'])
        self.assertIn(b'return real_lookup(key, defl);', selected['pxt-config.cpp'])
        self.assertNotIn(b'ignored', selected['pxt-config.cpp'])
        self.assertNotIn(b'ignored', selected['pxt-fiber-roots.cpp'])
        self.assertNotIn(b' malloc(', selected['target-native.cpp'])
        self.assertNotIn(b' free(', selected['target-native.cpp'])
        self.assertEqual(selected['target-native.cpp'].count(b'bw_native_free('), 4)
        self.assertIn(gc.REALLOC_PAYLOAD_COPY, selected['gc-storage.cpp'])
        self.assertNotIn(gc.REALLOC_COPY, selected['gc-storage.cpp'])
        # Removing only the declared attribute must restore the native-domain
        # adapted target byte-for-byte; no clearing algorithm is substituted.
        target = (gc.ROOT / 'runtime-services/target.cpp').read_bytes()
        native = gc.native_calls(target, 2, 4, b'#include "Runtime.h"\n')
        self.assertEqual(selected['target-native.cpp'].replace(
            b'__attribute__((optimize("O2"))) ', b''), native)

    def test_realloc_copy_site_refuses_missing_or_repeated(self):
        for changed in [b'', gc.REALLOC_COPY * 2]:
            inputs = payloads()
            inputs['pxtapp/base/gc.cpp'] = inputs['pxtapp/base/gc.cpp'].replace(
                gc.REALLOC_COPY, changed)
            with self.assertRaises(ValueError):
                gc.adapt(inputs, (gc.ROOT / 'runtime-services/target.cpp').read_bytes())

    def test_changed_native_allocation_census_refuses(self):
        target = (gc.ROOT / 'runtime-services/target.cpp').read_bytes()
        for changed in [target + b'\nvoid extra() { malloc(1); }\n',
                        target.replace(b'free(memory)', b'other(memory)', 1)]:
            with self.assertRaises(ValueError):
                gc.adapt(payloads(), changed)

    def test_native_call_domains_preserve_other_bytes(self):
        raw = b'#include "Timer.h"\n// retained body\nvoid *f() { return malloc(12); }\n'
        changed = gc.native_calls(raw, 1, 0, b'#include "Timer.h"\n')
        self.assertEqual(changed, raw.replace(b'#include "Timer.h"\n',
                         b'#include "Timer.h"\n#include "native-aligned-heap.h"\n')
                         .replace(b'malloc(', b'bw_native_alloc('))
        with self.assertRaises(ValueError):
            gc.native_calls(raw + b'\nfree(p);\n', 1, 0, b'#include "Timer.h"\n')

    def test_no_work_before_hosted_admission(self):
        with mock.patch.dict(gc.consumer.os.environ, {'GITHUB_ACTIONS': 'false'}), \
             mock.patch.object(gc.subprocess, 'run') as command:
            with self.assertRaisesRegex(ValueError, 'hosted CI only'):
                gc.run(Path('core'), Path('framework'), Path('generator'), Path('out'))
            command.assert_not_called()

    def test_static_constructor_retention_and_invocation(self):
        startup = (gc.ROOT / 'pxt-gc-runtime/startup.S').read_text()
        linker = (gc.ROOT / 'pxt-gc-runtime/control.ld').read_text()
        self.assertIn('KEEP(*(.init_array))', linker)
        self.assertIn('ldr r4, =_init_array_start', startup)
        self.assertIn('ldr r5, =_init_array_end', startup)
        self.assertLess(startup.index('blx r3'), startup.index('bl runtime_probe'))

    def test_failure_artifact_notices_precede_source_staging(self):
        source = Path(gc.__file__).read_text()
        self.assertLess(source.index("(notices / 'LICENSE-pxt-common-packages').write_bytes"),
                        source.index('for name, raw in staged.items():'))


if __name__ == '__main__':
    unittest.main()
