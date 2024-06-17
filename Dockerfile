FROM rust:1.79-alpine as builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apk add git build-base ca-certificates musl-dev openssl-dev

WORKDIR /usr/src/app

COPY . .

ENV RUSTFLAGS="-Ctarget-feature=-crt-static"

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
ENV RUSTFLAGS="-Ctarget-feature=-crt-static"

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/relayer ./
CMD [ "./relayer" ]
