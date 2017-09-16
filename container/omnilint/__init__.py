'''omnilint main entry point'''

import os
import re
import importlib


class Omnilint(object):
    '''Main omnilint backend class'''

    exec_regexes = [
        re.compile('^#!/usr/bin/env ([^ ]+)'),
        re.compile('^#!/bin/env ([^ ]+)'),
        re.compile('^#! *([^ ]+)'),
    ]

    def __init__(self):
        self.checkers = []

    def register(self, checker):
        self.checkers.append(checker)

    def checkers_load(self):
        checkers_directory = os.path.join(os.path.dirname(__file__), 'checker')
        for filename in os.listdir(checkers_directory):
            if filename.startswith('__'):
                continue
            self.checker_load(os.path.splitext(filename)[0])

    def checker_load(self, checker_name):
        module_name = 'omnilint.checker.' + checker_name
        checker = importlib.import_module(module_name)
        checker.register(self)

    def analyse_file(self, reporter, filename):
        with open(filename) as fd:
            try:
                firstline = fd.readline().rstrip()
            except:
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
            with open(filename) as fd:
                for p in self.checkers:
                    if (executable and executable in p.executables) \
                       or (extension and extension in p.extensions):
                        c = p()
                        fd.seek(0)
                        c.check(
                            reporter,
                            origname=filename,
                            tmpname=filename,
                            firstline=firstline,
                            fd=fd)
