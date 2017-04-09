'''Collection of reporter classes'''


class ReporterLine(object):
    '''Default reporter object with human-readable line-oriented
    format (gcc)'''

    def __init__(self, fd):
        self.fd = fd
        self.num_errors = 0

    def report(self, error):
        self.fd.write(error.gcc_style())
        self.fd.write('\n')
        self.num_errors += 1
