# This file is subject to the terms and conditions defined in
# file 'LICENSE', which is part of this source code package.
'''Useful functions for checkers, and base class'''


class Checker(object):
    '''A base class for checkers'''

    executables = []
    extensions = []
    mimetypes = []
