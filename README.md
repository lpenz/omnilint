# omnilint

**omnilint** is a container-based tool that provides a unified interface for the
static analysis of any file.

    File                    Linter
    .c  \                  / cppcheck
    .sh  -    omnilint    -  shelcheck
    .py /        |         \ pyflakes
            json output

