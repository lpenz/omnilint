'''Test checkers'''

import unittest

import omnilint_test_common as common


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

    def test_python_flake8(self):
        errs = common.checkthis('flake8', '.py', 'import re\n')
        self.assertEqual([dict(e) for e in errs], [{
            'line': 1, 'column': 1,
            'message': "F401 're' imported but unused"}])
