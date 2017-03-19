'''Error class'''


class Error(object):
    def __init__(self, msg, file=None, line=None, column=None):
        self.msg = msg
        self.file = file
        self.line = line
        self.column = column

    def gcc_style(self):
        output = []
        if self.file is not None:
            output.append(self.file + ':')
            if self.line is not None:
                output.append(str(self.line) + ':')
                if self.column is not None:
                    output.append(str(self.column) + ':')
            output.append(' ')
        output.append(self.msg)
        return ''.join(output)
