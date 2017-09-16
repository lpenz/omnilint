'''python flake8 checker'''

import subprocess
import re

from omnilint.error import Error
from omnilint.checker import Checker


class PythonFlake8(Checker):

    executables = ['python', 'python3']

    def __init__(self):
        super(PythonFlake8, self).__init__()

    def check(self, reporter, origname, tmpname, firstline, fd):
        p = subprocess.Popen(['flake8', tmpname], stdout=subprocess.PIPE)
        regex = re.compile(''.join([
            '^(?P<path>[^:]+)', ':(?P<line>[0-9]+)', ':(?P<column>[0-9]+)',
            ': (?P<message>.*)$'
        ]))

        for l in p.stdout:
            m = regex.match(l.rstrip())
            assert m
            reporter.report(
                Error(
                    msg=m['message'],
                    file=origname,
                    line=m['line'],
                    column=m['column'], ))

        p.wait()


def register(omnilint):
    '''Registration function, called by omnilint while loading the checker with
    itself as argument'''
    omnilint.register(PythonFlake8)
