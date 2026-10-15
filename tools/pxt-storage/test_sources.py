# SPDX-License-Identifier: MIT
import copy
import json
from pathlib import Path
import tempfile
import unittest

import sources


class SourceControls(unittest.TestCase):
    def setUp(self):
        self.profile = sources.load(sources.ROOT / 'profiles/pxt-framework-sources.json')

    def test_positive_closed_public_mapping(self):
        sources.validate(self.profile)
        self.assertEqual(len(self.profile['sources']), 64)
        self.assertEqual(len(self.profile['unresolved']), 5)

    def test_qualification_cannot_be_broadened(self):
        for flag in sources.FALSE_FLAGS:
            for value in [True, 0, None, 'false']:
                with self.subTest(flag=flag, value=value):
                    p = copy.deepcopy(self.profile)
                    p[flag] = value
                    with self.assertRaises(ValueError):
                        sources.validate(p)

    def test_mapping_refusals(self):
        mutations = [lambda p: p.update(schema=True),
                     lambda p: p.update(runtime_qualified=True),
                     lambda p: p['package'].update(commit='0' * 40),
                     lambda p: p['package'].update(notice_git_blob='0' * 40),
                     lambda p: p['sources'].pop(),
                     lambda p: p['sources'].__setitem__(1, dict(p['sources'][0])),
                     lambda p: p['sources'][0].update(request='/pxtapp/../escape.h'),
                     lambda p: p['sources'][0].update(path='libs/../../escape.h'),
                     lambda p: p['sources'][0].update(sha256='0' * 64),
                     lambda p: p['sources'][0].update(bytes=True),
                     lambda p: p['unresolved'].pop(),
                     lambda p: p['unresolved'][0].update(status='QUALIFIED'),
                     lambda p: p['archive'].update(request_sha256='0' * 64)]
        for i, mutate in enumerate(mutations):
            with self.subTest(i=i):
                p = copy.deepcopy(self.profile)
                mutate(p)
                with self.assertRaises(ValueError):
                    sources.validate(p)

    def test_duplicate_json(self):
        with self.assertRaises(ValueError):
            json.loads('{"a":1,"a":2}', object_pairs_hook=sources.pairs)

    def test_path_refusals(self):
        for path in ['/absolute', '../escape', 'a/../b', 'a\\b', 'a//b', './a', '']:
            with self.subTest(path=path), self.assertRaises(ValueError):
                sources.safe(path)

    def test_stage_refuses_before_writing(self):
        with tempfile.TemporaryDirectory(prefix='pxt-source-controls-') as temporary:
            target = Path(temporary) / 'new'
            with self.assertRaisesRegex(ValueError, 'source census'):
                sources.stage(self.profile, {}, b'not a notice', target)
            self.assertFalse(target.exists())
            with self.assertRaisesRegex(ValueError, 'new destination'):
                sources.stage(self.profile, {}, b'not a notice', Path(temporary))

    def test_no_private_paths_or_bodies(self):
        text = json.dumps(self.profile)
        self.assertNotRegex(text, r'/(?:home|mnt|tmp|Users)/')
        for row in self.profile['sources']:
            self.assertEqual(set(row), {'request', 'path', 'git_blob', 'sha256', 'bytes'})


if __name__ == '__main__':
    unittest.main()
