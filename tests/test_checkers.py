# Copyright (C) 2017 Leandro Lisboa Penz <lpenz@lpenz.org>
# This file is subject to the terms and conditions defined in
# file 'LICENSE', which is part of this source code package.
'''Test checkers'''

import unittest

import omnilint_test_common as common


class TestCheckers(unittest.TestCase):
    def test_json(self):
        errs = common.checkthis(
            'json', '.json', '''
{
    "x": "asdf",
    "y": 9,
    "z": false
}
        ''')
        self.assertEqual(errs, [])
        errs = common.checkthis(
            'json', '.json', '''
{
    "x": "asdf",
    "y": 9,
}
        ''')
        self.assertEqual(
            [dict(e) for e in errs],
            [{
                'line': 5,
                'column': 1,
                'message': 'Expecting property name enclosed in double quotes'
            }])

    def test_python_flake8(self):
        errs = common.checkthis('python.flake8', '.py', 'import re\n')
        self.assertEqual([dict(e) for e in errs],
                         [{
                             'line': 1,
                             'column': 1,
                             'message': "F401 're' imported but unused"
                         }])

    def test_perl(self):
        errs = common.checkthis('perl', '.pl', '''
print "asdf\n";
        ''')
        self.assertEqual([dict(e) for e in errs], [
            {
                'line': 2,
                'column': 1,
                'message': 'Module does not end with "1;"'
            },
            {
                'line': 2,
                'column': 1,
                'message': 'Code not contained in explicit package'
            },
            {
                'line': 2,
                'column': 1,
                'message': 'Code before strictures are enabled'
            },
            {
                'line': 2,
                'column': 1,
                'message': 'Code before warnings are enabled'
            },
            {
                'line': 2,
                'column': 7,
                'message': 'Literal line breaks in a string'
            },
        ])
        errs = common.checkthis(
            'perl', '.pl', '''#!/usr/bin/perl

use warnings;
use strict;

print "asdf\\n";

1;
            ''')
        self.assertEqual(errs, [])
