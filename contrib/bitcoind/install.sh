#!/bin/bash

set -e


decho(){
    1>&2 echo $@
}

install_dep(){
    apt-get update
    apt-get install -y wget
}

install_bitcoind(){
    TMPDIR=$(mktemp --dir)
    cd $TMPDIR
    wget https://download.bitcoinsv.io/bitcoinsv/1.0.5/bitcoin-sv-1.0.5-x86_64-linux-gnu.tar.gz
    sha256sum bitcoin-sv-1.0.5-x86_64-linux-gnu.tar.gz | grep 96f7c56c7ebd4ecb2dcd664297fcf0511169ac33eaf216407ebc49dae2535578
    tar xvf bitcoin-sv-1.0.5-x86_64-linux-gnu.tar.gz
    ln -s bitcoin-sv-1.0.5 bitcoin
    install -m 0755 bitcoin/bin/* /usr/local/bin
    chmod +x /usr/local/bin/*   
    rm -rf $TMPDIR
}

install_conf(){
    mkdir -p /etc/bitcoind/
    cp bitcoind.conf /etc/bitcoind/bitcoind.conf
}

install_run(){
    install -m 0755 run.sh /usr/local/bin
}

user_create(){
    useradd -ms /bin/bash bitcoinduser
    mkdir /home/bitcoinduser/data
    chown -R bitcoinduser:bitcoinduser /home/bitcoinduser/data
}


case $1 in
    dep)
        ( install_dep )
    ;;
    bitcoind)
        ( install_bitcoind )
    ;;
    run)
        ( install_run )
    ;;
    user)
        ( user_create )
    ;;
    conf)
        ( install_conf )
    ;;
esac