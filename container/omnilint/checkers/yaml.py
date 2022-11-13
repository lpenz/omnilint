# Copyright (C) 2017 Leandro Lisboa Penz <lpenz@lpenz.org>
# This file is subject to the terms and conditions defined in
# file 'LICENSE', which is part of this source code package.
"""yaml checker"""

from omnilint.checkers import Checker
from omnilint.error import Error

import yaml


class Yaml(Checker):

    extensions = ["yaml", "yml"]

    def __init__(self):
        super(Yaml, self).__init__()

    def check(self, reporter, origname, tmpname, firstline, fd):
        exc = None
        try:
            yaml.load(fd, yaml.FullLoader)
        except yaml.YAMLError as e:
            exc = e
        if exc is None:
            return
        reporter.report(
            Error(
                msg=exc.context + " " + exc.problem,
                file=origname,
                line=exc.problem_mark.line,
                column=exc.problem_mark.column,
            )
        )


def register(omnilint):
    """Registration function, called by omnilint while loading the checker with
    itself as argument"""
    omnilint.register(Yaml)
