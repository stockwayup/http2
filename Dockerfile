FROM rust:1.89.0-alpine3.20 AS builder

RUN apk add --no-cache musl-dev

COPY . /tmp/rust/src/github.com/stockwayup/http2

WORKDIR /tmp/rust/src/github.com/stockwayup/http2

RUN cargo build --release

FROM alpine:3.20

RUN adduser -S www-data -G www-data

COPY --from=builder --chown=www-data /tmp/rust/src/github.com/stockwayup/http2/target/release/http2 /bin/http2

RUN chmod +x /bin/http2

USER www-data

CMD ["/bin/http2"]
