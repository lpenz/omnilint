# Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
# This file is subject to the terms and conditions defined in
# file 'LICENSE', which is part of this source code package.
'''ansible playbook checker'''

import yaml
import subprocess
import re

from omnilint.error import Error
from omnilint.checkers import Checker


class AnsiblePlaybook(Checker):

    extensions = ['yaml', 'yml']

    def __init__(self):
        super(AnsiblePlaybook, self).__init__()

    def isplaybook(self, data):
        if not isinstance(data, list):
            return False
        for e in data:
            if not isinstance(e, dict):
                return False
            if 'import_playbook' in e:
                return True
            if 'hosts' in e:
                return True
        return False

    def check(self, reporter, origname, tmpname, firstline, fd):
        try:
            data = yaml.load(fd)
        except yaml.YAMLError:
            # This is reported by the yaml checker:
            return
        if not self.isplaybook(data):
            return
        with subprocess.Popen(
            ['/usr/local/bin/ansible-lint', '-p', '--nocolor', tmpname],
                stdout=subprocess.PIPE,
                env={'HOME': '/tmp'}) as p:
            regex = re.compile(''.join([
                '^(?P<path>[^:]+)', ':(?P<line>[0-9]+)', ': (?P<message>.*)$'
            ]))

            for line in p.stdout:
                line = line.decode('utf-8').rstrip()
                m = regex.match(line)
                assert m
                reporter.report(
                    Error(msg=m.group('message'),
                          file=m.group('path'),
                          line=int(m.group('line'))))


def register(omnilint):
    '''Registration function, called by omnilint while loading the checker with
    itself as argument'''
    omnilint.register(AnsiblePlaybook)
