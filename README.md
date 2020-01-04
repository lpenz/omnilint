[![Build Status](https://travis-ci.org/lpenz/omnilint.svg?branch=master)](https://travis-ci.org/lpenz/omnilint)
[![Docker Automated build](https://img.shields.io/docker/automated/lpenz/omnilint.svg)](https://hub.docker.com/r/lpenz/omnilint/builds/)

# omnilint


## TL;DR

Add the following to your *.travis.yml* to get a job that performs static
analysis on the files of your repository:

```yaml
jobs:
  include:
    - name: omnilint
      language: generic
      install: docker pull lpenz/omnilint
      script: docker run --rm -u "$UID" -v "$PWD:$PWD" -w "$PWD" lpenz/omnilint
```

Or use it as a github action with a
[workflow](https://help.github.com/en/actions/automating-your-workflow-with-github-actions/configuring-a-workflow)
file like the following:

```yaml
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@master
      - uses: docker://lpenz/omnilint:latest
```


## What is omnilint

**omnilint** is a container-based tool that provides a unified interface for the
static analysis of any file, using the appropriate tool.

    File                    Linter
    .c  \                  / cppcheck
    .sh  -    omnilint    -  shelcheck
    .py /        |         \ pyflakes
          gcc-like output
            json output


omnilint is specially useful when combined with CI tools
like [travis](https://travis-ci.org), [jenkins](https://jenkins.io), etc.


## Installation and local usage

Requirements:
- docker


The easier way to install omnilint is by pulling the container from
[docker hub](https://hub.docker.com/r/lpenz/omnilint/):

    docker pull lpenz/omnilint

Using omnilint is a matter of starting the container with the directory with the
files to be analyzed mapped:

    docker run --rm -u "$UID" -v "$PWD:$PWD" -w "$PWD" omnilint

You can also simply call the script `bin/omnilint-docker-run` that is present in
the repository.

