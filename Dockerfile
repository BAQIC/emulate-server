FROM rust:alpine AS builder

RUN mkdir -p /workspace
WORKDIR /workspace

RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.bfsu.edu.cn/g' /etc/apk/repositories \
    && apk add git musl-dev make perl --no-cache

WORKDIR /workspace/emulate-server
COPY . .
RUN cargo build --release && mv target/release/emulate-server /bin/emulate-server \
    && cargo clean && rm -rf /usr/local/cargo \
    && rm -rf /usr/local/rustup

FROM alpine:latest
RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.bfsu.edu.cn/g' /etc/apk/repositories \
    && apk add musl --no-cache
COPY --from=builder /workspace/emulate-server/config/qsched.json /qsched.json
COPY --from=builder /workspace/emulate-server/config/log4rs.yaml /log4rs.yaml
COPY --from=builder /bin/emulate-server /bin/emulate-server

ENTRYPOINT [ "/bin/emulate-server" ]
