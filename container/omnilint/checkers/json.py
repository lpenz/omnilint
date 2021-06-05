# Copyright (C) 2017 Leandro Lisboa Penz <lpenz@lpenz.org>
# This file is subject to the terms and conditions defined in
# file 'LICENSE', which is part of this source code package.
'''json checker'''

import json

from omnilint.error import Error
from omnilint.checkers import Checker


class Json(Checker):

    extensions = ['json']

    def __init__(self):
        super(Json, self).__init__()

    def check(self, reporter, origname, tmpname, firstline, fd):
        exc = None
        try:
            json.load(fd)
        except json.JSONDecodeError as e:
            exc = e
        if exc is None:
            return
        reporter.report(
            Error(msg=exc.msg,
                  file=origname,
                  line=exc.lineno,
                  column=exc.colno))


def register(omnilint):
    '''Registration function, called by omnilint while loading the checker with
    itself as argument'''
    omnilint.register(Json)
