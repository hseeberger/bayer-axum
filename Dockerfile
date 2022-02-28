FROM rust:1.59.0-bullseye AS builder
WORKDIR /usr/src/bayer-axum
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/bayer-axum /usr/local/bin/bayer-axum
COPY --from=builder /usr/src/bayer-axum/config /opt/bayer-axum/config
WORKDIR /opt/bayer-axum
ENTRYPOINT ["bayer-axum"]
