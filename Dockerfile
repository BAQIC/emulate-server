FROM rust:alpine

RUN mkdir -p /workspace
WORKDIR /workspace

RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.bfsu.edu.cn/g' /etc/apk/repositories \
    && apk add git && apk add musl-dev make perl \
    && git clone https://github.com/BAQIC/emulate-server.git

WORKDIR /workspace/emulate-server
RUN cargo build

ENTRYPOINT [ "cargo", "run" ]
