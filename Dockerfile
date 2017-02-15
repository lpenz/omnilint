FROM debian:stretch
MAINTAINER Leandro Penz <lpenz@lpenz.org>

# install debian packages:
ENV DEBIAN_FRONTEND=noninteractive
RUN set -x -e; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        # shell:
        shellcheck \
        # python:
        flake8 python-setuptools python-pip python-wheel \
        # base packages:
        locales gosu sudo

# setup sudo and locale
RUN set -x -e; \
    echo 'ALL ALL=NOPASSWD:ALL' > /etc/sudoers.d/all; \
    chmod 0400 /etc/sudoers.d/all; \
    echo 'en_US.UTF-8 UTF-8' >> /etc/locale.gen; \
    locale-gen
ENV LC_ALL=en_US.UTF-8

# install pip packages:
RUN set -x -e; \
    pip install \
        py3kwarn==0.4.4

COPY container/omnilint /usr/local/bin/

# setup entrypoint with user UID/GID from host
RUN set -x -e; \
    (\
    echo '#!/bin/bash'; \
    echo 'MY_UID=${MY_UID:-1000}'; \
    echo 'set -x -e'; \
    echo 'useradd -M -u "$MY_UID" -o user'; \
    echo 'chown user:user /home/user'; \
    echo 'cd /home/user/work'; \
    echo 'exec gosu user "${@:-/bin/bash}"'; \
    ) > /usr/local/bin/entrypoint.sh; \
    chmod a+x /usr/local/bin/entrypoint.sh
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

CMD ["/usr/local/bin/omnilint","/home/user/work"]

# If your UID is 1000, you can simply run the container as
# docker run -it --rm -v $PWD:/home/user/work ${PWD##*/}
# otherwise, run it as:
# docker run -it --rm -v $PWD:/home/user/work -e MY_UID=$UID ${PWD##*/}
