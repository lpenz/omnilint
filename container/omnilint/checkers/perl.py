# Copyright (C) 2018 Leandro Lisboa Penz <lpenz@lpenz.org>
# This file is subject to the terms and conditions defined in
# file 'LICENSE', which is part of this source code package.
'''perl checker'''

import subprocess
import re

from omnilint.error import Error
from omnilint.checkers import Checker


class Perl(Checker):

    executables = ['perl']
    extensions = ['pl', 'perl']

    def __init__(self):
        super(Perl, self).__init__()

    def check(self, reporter, origname, tmpname, firstline, fd):
        with subprocess.Popen(
            ['perlcritic', '--verbose', '1', '--harsh', tmpname],
                stdout=subprocess.PIPE) as p:
            regex = re.compile(''.join([
                '^(?P<path>[^:]+)', ':(?P<line>[0-9]+)', ':(?P<column>[0-9]+)',
                ':(?P<message>.*)$'
            ]))

            linenum = 0
            sourceok = bytes(tmpname, 'ascii') + b' source OK\n'
            foundsourceok = False
            for line in p.stdout:
                if linenum == 0 and line == sourceok:
                    foundsourceok = True
                    continue
                assert not foundsourceok
                line = line.decode('utf-8').rstrip()
                m = regex.match(line)
                assert m, line
                reporter.report(
                    Error(msg=m.group('message'),
                          file=origname,
                          line=int(m.group('line')),
                          column=int(m.group('column'))))
                linenum += 1


def register(omnilint):
    '''Registration function, called by omnilint while loading the checker with
    itself as argument'''
    omnilint.register(Perl)
