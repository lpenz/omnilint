# This file is subject to the terms and conditions defined in
# file 'LICENSE', which is part of this source code package.
'''Basic tests'''

import unittest

import omnilint


class TestBasic(unittest.TestCase):

    def test_basic(self):
        ol = omnilint.Omnilint()
        ol.checkers_load()
        self.assertNotEqual(ol.checkers, False)
