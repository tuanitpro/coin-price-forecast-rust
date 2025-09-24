FROM rust:latest AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    musl-tools \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl || true

COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest
WORKDIR /app

# dynamic OpenSSL needed here
RUN apk add --no-cache libssl3

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/main ./main
CMD ["./main"]
