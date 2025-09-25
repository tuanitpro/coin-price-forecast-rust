FROM rust:latest AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    musl-tools \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY . .
RUN cargo build --release

FROM alpine:3.20
WORKDIR /bin

# dynamic OpenSSL needed here
RUN apk add --no-cache libssl3

COPY --from=builder /app/target/release/main .
CMD ["./main"]
