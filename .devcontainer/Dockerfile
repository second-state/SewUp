FROM wasmedge/wasmedge:ubuntu-build-clang
MAINTAINER yanganto@gmail.com

RUN apt update && apt install -y \
	cmake pkg-config openssl libssl-dev build-essential curl
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -sSf | bash -s -- -y
RUN /bin/bash -c "source $HOME/.cargo/env \
    && rustup default nightly-2021-11-29 \
    && rustup target add wasm32-unknown-unknown \
    && cargo install cargo-sewup"
