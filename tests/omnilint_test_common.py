# Copyright (C) 2017 Leandro Lisboa Penz <lpenz@lpenz.org>
# This file is subject to the terms and conditions defined in
# file 'LICENSE', which is part of this source code package.
'''Common functions used by tests'''

import tempfile

import omnilint
import omnilint.reporters as reporters


def checkthis(checkername, fileext, contents):
    ol = omnilint.Omnilint()
    ol.checker_load(checkername)
    with tempfile.NamedTemporaryFile(suffix=fileext) as tmp:
        tmp.write(contents.encode('utf-8'))
        tmp.flush()
        with reporters.ReporterList() as reporter:
            ol.analyse_file(reporter, tmp.name)
            for e in reporter.list:
                e.pop('file')
            return reporter.list
