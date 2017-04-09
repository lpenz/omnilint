'''Collection of reporter classes'''

from collections import OrderedDict
import json


class Reporter(object):
    def __init__(self, fd):
        self.fd = fd
        self.num_errors = 0

    def report(self, error):
        self.num_errors += 1

    def close(self):
        pass


class ReporterGcc(Reporter):
    '''Default reporter object with human-readable line-oriented
    format (gcc)'''

    def __init__(self, fd):
        super(ReporterGcc, self).__init__(fd)

    def report(self, error):
        super(ReporterGcc, self).report(error)
        self.fd.write(error.gcc_style())
        self.fd.write('\n')
        self.num_errors += 1


class ReporterJsonList(Reporter):
    '''Reporter that writes a json list of error dictionaries'''

    def __init__(self, fd):
        super(ReporterJsonList, self).__init__(fd)
        self.fd.write('[\n')

    def report(self, error):
        super(ReporterJsonList, self).report(error)
        self.fd.write(' ' * 2)
        json.dump(OrderedDict(error), self.fd)
        self.fd.write('\n')

    def close(self):
        self.fd.write(']\n')


REPORTERS = {
    'gcc': ReporterGcc,
    'json-list': ReporterJsonList,
}
