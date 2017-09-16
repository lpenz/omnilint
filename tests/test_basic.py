'''Basic tests'''

import unittest

import omnilint

import omnilint_test_common as common


class TestBasic(unittest.TestCase):

    def test_basic(self):
        ol = omnilint.Omnilint()
        ol.checkers_load()
        self.assertNotEqual(ol.checkers, False)


class TestCheckers(unittest.TestCase):

    def test_json(self):
        errs = common.checkthis('json', '.json', '''
{
    "x": "asdf",
    "y": 9,
    "z": false
}
        ''')
        self.assertEqual(errs, [])
        errs = common.checkthis('json', '.json', '''
{
    "x": "asdf",
    "y": 9,
}
        ''')
        self.assertEqual([dict(e) for e in errs], [{
            'line': 5, 'column': 1,
            'message': 'Expecting property name enclosed in double quotes'}])
