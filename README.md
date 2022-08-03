[![marketplace](https://img.shields.io/badge/marketplace-omnilint-blue?logo=crosshair)](https://github.com/marketplace/actions/ghaction-omnilint)
[![CI](https://github.com/lpenz/omnilint/actions/workflows/ci.yml/badge.svg)](https://github.com/lpenz/omnilint/actions/workflows/ci.yml)
[![github](https://img.shields.io/github/v/release/lpenz/omnilint?include_prereleases&label=release&logo=github)](https://github.com/lpenz/omnilint/releases)
[![docker](https://img.shields.io/docker/v/lpenz/omnilint?label=release&logo=docker&sort=semver)](https://hub.docker.com/repository/docker/lpenz/omnilint)

# omnilint


## TL;DR

Use *omnilint* as a github action with a
[workflow](https://help.github.com/en/actions/automating-your-workflow-with-github-actions/configuring-a-workflow)
file like the following:

```yaml
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: docker://lpenz/omnilint:0.5
```

Another option is using this repository's callable workflow:

```yaml
name: CI
on: push
jobs:
  omnilint:
    uses: lpenz/ghworkflow-rust/.github/workflows/rust.yml@v0.5
```

Or add the following to your *.travis.yml* to get a job that performs
static analysis on the files of your repository:

```yaml
jobs:
  include:
    - name: omnilint
      language: generic
      install: docker pull lpenz/omnilint
      script: docker run --rm -u "$UID" -v "$PWD:$PWD" -w "$PWD" lpenz/omnilint
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

