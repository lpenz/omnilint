# Copyright (C) 2017 Leandro Lisboa Penz <lpenz@lpenz.org>
# This file is subject to the terms and conditions defined in
# file 'LICENSE', which is part of this source code package.
'''shellcheck checker'''

import subprocess
import json

from omnilint.error import Error
from omnilint.checkers import Checker


class Shell(Checker):

    executables = ['bash', 'sh', 'ash']

    def __init__(self):
        super(Shell, self).__init__()

    def check(self, reporter, origname, tmpname, firstline, fd):
        p = subprocess.Popen(['shellcheck', '-fjson', tmpname],
                             stdout=subprocess.PIPE)
        output = p.stdout.read().decode('utf-8')
        errors = json.loads(output)
        p.wait()
        for e in errors:
            reporter.report(
                Error(
                    msg=e['message'],
                    file=origname,
                    line=e['line'],
                    column=e['column'],
                ))


def register(omnilint):
    '''Registration function, called by omnilint while loading the checker with
    itself as argument'''
    omnilint.register(Shell)
