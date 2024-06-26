FROM rust:1.79-alpine as builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apk add git build-base ca-certificates

WORKDIR /usr/src/app

COPY . .

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
COPY --from=alpine:3.9 /bin/sh /bin/sh
COPY --from=alpine:3.9 /usr /usr
COPY --from=alpine:3.9 /lib /lib

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/relayer ./
CMD [ "./relayer" ]
