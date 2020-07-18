# This file is subject to the terms and conditions defined in
# file 'LICENSE', which is part of this source code package.
'''python flake8 checker'''

import subprocess
import re

from omnilint.error import Error
from omnilint.checkers import Checker
from omnilint.checkers.python import PythonFiletype


class PythonFlake8(PythonFiletype, Checker):
    def __init__(self):
        Checker.__init__(self)

    def check(self, reporter, origname, tmpname, firstline, fd):
        with subprocess.Popen(['flake8', tmpname],
                              stdout=subprocess.PIPE) as p:
            regex = re.compile(''.join([
                '^(?P<path>[^:]+)', ':(?P<line>[0-9]+)', ':(?P<column>[0-9]+)',
                ': (?P<message>.*)$'
            ]))

            for line in p.stdout:
                line = line.decode('utf-8').rstrip()
                m = regex.match(line)
                assert m
                reporter.report(
                    Error(msg=m.group('message'),
                          file=origname,
                          line=int(m.group('line')),
                          column=int(m.group('column'))))


def register(omnilint):
    '''Registration function, called by omnilint while loading the checker with
    itself as argument'''
    omnilint.register(PythonFlake8)
