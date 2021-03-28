FROM rustlang/rust:nightly as builder
WORKDIR /usr/src/app
COPY . .
# RUN cargo install cargo-travis
RUN export PATH=$HOME/.cargo/bin:$PATH
# RUN cargo rustc -- -Awarnings
RUN cargo build --all --verbose --color always
# RUN cargo coveralls --exclude-pattern tests/,script/,bitcoin/src/,encode-derive/ --all