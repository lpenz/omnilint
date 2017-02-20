'''Collection of reporter classes'''

import sys


class ReporterStderr(object):
    '''Default reporter object that simply writes errors to stderr
    in gcc format'''

    def __init__(self):
        self.num_errors = 0

    def report(self, msg, file=None, line=None, column=None):
        output = []
        if file is not None:
            output.append(file + ':')
            if line is not None:
                output.append(str(line) + ':')
                if column is not None:
                    output.append(str(column) + ':')
            output.append(' ')
        output.append(msg)
        output.append('\n')
        sys.stderr.write(''.join(output))
        self.num_errors += 1
