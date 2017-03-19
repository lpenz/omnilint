'''Collection of reporter classes'''

import sys


class ReporterStderr(object):
    '''Default reporter object that simply writes errors to stderr
    in gcc format'''

    def __init__(self):
        self.num_errors = 0

    def report(self, error):
        sys.stderr.write(error.gcc_style())
        sys.stderr.write('\n')
        self.num_errors += 1
