FROM rust:alpine

RUN mkdir -p /workspace
WORKDIR /workspace

# Change the source of crates.io to TUNA
RUN touch $CARGO_HOME/config \
    && echo '[source.crates-io]' > $CARGO_HOME/config \
    && echo 'registry = "https://github.com/rust-lang/crates.io-index"'  >> $CARGO_HOME/config \
    && echo "replace-with = 'tuna'"  >> $CARGO_HOME/config \
    && echo '[source.tuna]'   >> $CARGO_HOME/config \
    && echo 'registry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"'  >> $CARGO_HOME/config \
    && echo '[net]'   >> $CARGO_HOME/config \
    && echo 'git-fetch-with-cli = true'   >> $CARGO_HOME/config \
    && echo '' >> $CARGO_HOME/config

RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.bfsu.edu.cn/g' /etc/apk/repositories \
    && apk add git && apk add musl-dev \
    && git clone https://github.com/BAQIC/emulate-server.git

WORKDIR /workspace/emulate-server
RUN cargo fetch

ENTRYPOINT [ "cargo", "run" ]
