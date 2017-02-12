[![Build Status](https://travis-ci.org/lpenz/omnilint.svg?branch=master)](https://travis-ci.org/lpenz/omnilint)

# omnilint

**omnilint** is a container-based tool that provides a unified interface for the
static analysis of any file, using the appropriate tool.

    File                    Linter
    .c  \                  / cppcheck
    .sh  -    omnilint    -  shelcheck
    .py /        |         \ pyflakes
            json output

