FROM rust:alpine

RUN mkdir -p /workspace
WORKDIR /workspace

RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.bfsu.edu.cn/g' /etc/apk/repositories \
    && apk add git musl-dev make perl --no-cache

WORKDIR /workspace/emulate-server
COPY . .
RUN cargo build --release && mv target/release/emulate-server /bin/emulate-server && cargo clean && rm -rf /usr/local/cargo && rm -rf /usr/local/rustup

ENTRYPOINT [ "/bin/emulate-server" ]
