#!/bin/bash


decho(){
    1>&2 echo $@
}

run_daemon(){
    exec /usr/local/bin/bitcoind -conf=/etc/bitcoind/bitcoind.conf -datadir=/home/bitcoinduser/data -daemon
}

run_daemon