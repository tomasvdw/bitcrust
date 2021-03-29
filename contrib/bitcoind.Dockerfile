FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

COPY contrib/bitcoind /usr/src/bitcoind

WORKDIR /usr/src/bitcoind

RUN /bin/bash /usr/src/bitcoind/install.sh dep
RUN /bin/bash /usr/src/bitcoind/install.sh bitcoind
RUN /bin/bash /usr/src/bitcoind/install.sh run
RUN /bin/bash /usr/src/bitcoind/install.sh user
RUN /bin/bash /usr/src/bitcoind/install.sh conf

WORKDIR /home/bitcoinduser

USER bitcoinduser

CMD ["/usr/local/bin/run.sh"]