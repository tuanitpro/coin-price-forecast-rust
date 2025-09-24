# Build stage
FROM rust:latest AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*
    
RUN rustup target add x86_64-unknown-linux-musl
# Copy Cargo.toml and Cargo.lock first (for caching dependencies)
COPY Cargo.toml Cargo.lock ./

# Create an empty main.rs to force Cargo to build dependencies first
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release || true

COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:latest
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/main ./main

RUN apk add --no-cache libssl3
CMD ["./main"]