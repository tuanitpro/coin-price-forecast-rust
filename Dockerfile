FROM rust:latest AS builder
WORKDIR /app

# musl target
RUN rustup target add x86_64-unknown-linux-musl

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies with vendored OpenSSL
RUN cargo build --release --target x86_64-unknown-linux-musl || true

# Copy sources
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:latest
WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/main ./main

CMD ["./main"]
