'''omnilint main entry point'''

import os
import re
import importlib


class Omnilint(object):
    '''Main omnilint backend class'''

    exec_regexes = [
        re.compile('^#! */usr/bin/env ([^ ]+)'),
        re.compile('^#! */bin/env ([^ ]+)'),
        re.compile('^#! *([^ ]+)'),
    ]

    def __init__(self):
        self.checkers = []

    def register(self, checker):
        self.checkers.append(checker)

    def checkers_load(self):
        omnilint_dir = os.path.dirname(__file__)
        checkers_dir = os.path.join(omnilint_dir, 'checkers')
        for root, dirs, files in os.walk(checkers_dir):
            for filename in files:
                if filename.startswith('__'):
                    continue
                modbasename = os.path.splitext(filename)[0]
                modprefix = os.path.relpath(root, checkers_dir)
                if modprefix == '.':
                    modprefix = ''
                else:
                    modprefix = modprefix.replace('/', '.') + '.'
                self.checker_load(modprefix + modbasename)

    def checker_load(self, checker_id):
        module_name = 'omnilint.checkers.' + checker_id
        checker = importlib.import_module(module_name)
        checker.register(self)

    def analyse_file(self, reporter, filename):
        with open(filename) as fd:
            try:
                firstline = fd.readline().rstrip()
            except Exception:
                firstline = None
            fd.seek(0)
            executable = None
            if firstline:
                for e in self.exec_regexes:
                    m = e.match(firstline)
                    if m:
                        executable = os.path.basename(m.group(1))
                        break
            extension = os.path.splitext(filename)[1]
            if extension and extension[0] == '.':
                extension = extension[1:]
            for p in self.checkers:
                if (executable and executable in p.executables) or (
                        extension and extension in p.extensions):
                    c = p()
                    fd.seek(0)
                    c.check(
                        reporter,
                        origname=filename,
                        tmpname=filename,
                        firstline=firstline,
                        fd=fd)
