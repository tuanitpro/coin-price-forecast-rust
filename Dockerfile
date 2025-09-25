FROM rust:slim AS builder
RUN apt-get update && apt-get install -y \
    musl-tools \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
    
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY . .
RUN cargo build --release

FROM alpine:3.22
WORKDIR /bin

# dynamic OpenSSL needed here
RUN apk add --no-cache libssl3

COPY --from=builder /app/target/release/main .
CMD ["./main"]
