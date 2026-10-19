# SPDX-License-Identifier: MIT
"""Pure admission failures; no compiler, guest or producer execution."""
import unittest
from build_storage import adapt,CORE_EDITS,RAFFS_EDITS,SETTINGS_EDITS

class AdmissionControls(unittest.TestCase):
    def test_unknown_or_duplicate_inputs_refuse(self):
        for edits in [CORE_EDITS,RAFFS_EDITS,SETTINGS_EDITS]:
            for missing,_ in edits:
                raw=b''.join(a for a,_ in edits if a!=missing)
                with self.assertRaises(ValueError):adapt(raw,edits)
            raw=b''.join(a for a,_ in edits)
            with self.assertRaises(ValueError):adapt(raw+edits[0][0],edits)

if __name__=='__main__':unittest.main()
