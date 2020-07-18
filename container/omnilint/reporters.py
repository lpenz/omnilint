# This file is subject to the terms and conditions defined in
# file 'LICENSE', which is part of this source code package.
'''Collection of reporter classes'''

from collections import OrderedDict
import json


class Reporter(object):
    def __init__(self):
        self.num_errors = 0

    def __enter__(self):
        return self

    def __exit__(self, type, value, traceback):
        pass

    def report(self, error):
        self.num_errors += 1


class ReporterGcc(Reporter):
    '''Default reporter object with human-readable line-oriented
    format (gcc)'''
    def __init__(self, fd):
        super(ReporterGcc, self).__init__()
        self.fd = fd

    def report(self, error):
        super(ReporterGcc, self).report(error)
        self.fd.write(error.gcc_style())
        self.fd.write('\n')


class ReporterJsonList(Reporter):
    '''Reporter that writes a json list of error dictionaries'''
    def __init__(self, fd):
        super(ReporterJsonList, self).__init__()
        self.fd = fd

    def __enter__(self):
        self.fd.write('[\n')

    def __exit__(self, type, value, traceback):
        self.fd.write(']\n')

    def report(self, error):
        super(ReporterJsonList, self).report(error)
        self.fd.write(' ' * 2)
        json.dump(OrderedDict(error), self.fd)
        self.fd.write('\n')


class ReporterList(Reporter):
    '''Reporter that stores the errors in an in-memory list'''
    def __init__(self):
        super(ReporterList, self).__init__()
        self.list = []

    def report(self, error):
        super(ReporterList, self).report(error)
        self.list.append(OrderedDict(error))
        return self.list


REPORTERS = {
    'gcc': ReporterGcc,
    'json-list': ReporterJsonList,
}
