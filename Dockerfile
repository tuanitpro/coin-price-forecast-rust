FROM rust:alpine AS builder
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    && rm -rf /var/cache/apk/*

RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --target x86_64-unknown-linux-musl --release

COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/main /usr/local/bin/main

ENTRYPOINT ["/usr/local/bin/main"]
