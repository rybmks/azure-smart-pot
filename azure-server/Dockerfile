FROM rust:latest as builder

WORKDIR /app

COPY Cargo.toml .
COPY Cargo.lock .

COPY src ./src

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /app

COPY --from=builder /app/target/release/azure-server /usr/local/bin/azure-server

CMD ["azure-server"]
