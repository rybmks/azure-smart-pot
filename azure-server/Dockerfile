FROM rust:latest

WORKDIR /app

COPY Cargo.toml .

COPY src ./src

RUN cargo build --release

CMD ["azure-server"]
