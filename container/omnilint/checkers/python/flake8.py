'''python flake8 checker'''

import subprocess
import re

from omnilint.error import Error
from omnilint.checkers import Checker


class PythonFlake8(Checker):

    executables = ['python', 'python3']
    extensions = ['py']

    def __init__(self):
        super(PythonFlake8, self).__init__()

    def check(self, reporter, origname, tmpname, firstline, fd):
        with subprocess.Popen(
                ['flake8', tmpname], stdout=subprocess.PIPE) as p:
            regex = re.compile(''.join([
                '^(?P<path>[^:]+)', ':(?P<line>[0-9]+)', ':(?P<column>[0-9]+)',
                ': (?P<message>.*)$'
            ]))

            for l in p.stdout:
                l = l.decode('utf-8').rstrip()
                m = regex.match(l)
                assert m
                reporter.report(
                    Error(
                        msg=m.group('message'),
                        file=origname,
                        line=int(m.group('line')),
                        column=int(m.group('column'))))


def register(omnilint):
    '''Registration function, called by omnilint while loading the checker with
    itself as argument'''
    omnilint.register(PythonFlake8)
