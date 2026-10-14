#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
import unittest
from check_docs import ROOT, validate


class Controls(unittest.TestCase):
    def test_existing_relative_file(self):
        self.assertEqual(validate(ROOT / "README.md", "[source](README.md)"), 1)

    def test_missing_file(self):
        with self.assertRaises(ValueError):
            validate(ROOT / "README.md", "[missing](missing-policy-control.md)")

    def test_path_escape(self):
        with self.assertRaises(ValueError):
            validate(ROOT / "README.md", "[escape](../../outside.md)")

    def test_absolute_path(self):
        with self.assertRaises(ValueError):
            validate(ROOT / "README.md", "[absolute](/README.md)")

    def test_private_path(self):
        with self.assertRaises(ValueError):
            validate(ROOT / "README.md", "/home/operator/evidence")

    def test_private_link_scheme(self):
        with self.assertRaises(ValueError):
            validate(ROOT / "README.md", "[private](file://machine/source)")

    def test_external_link_is_not_fetched_or_certified(self):
        self.assertEqual(validate(ROOT / "README.md", "[public](https://example.org/doc)"), 0)


if __name__ == "__main__":
    unittest.main()
