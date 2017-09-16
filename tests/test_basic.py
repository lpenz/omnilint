'''Tests basic loading'''

import unittest

import omnilint


class TestBasic(unittest.TestCase):

    def test_basic(self):
        ol = omnilint.Omnilint()
        ol.checkers_load()
        self.assertNotEqual(ol.checkers, False)
