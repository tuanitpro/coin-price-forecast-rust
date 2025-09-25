FROM rust:slim AS builder
RUN apt-get update && apt-get install -y \
    musl-tools \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --target x86_64-unknown-linux-musl --release

COPY src ./src
RUN cargo build --target x86_64-unknown-linux-musl --release


FROM scratch
WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/main /usr/local/bin/main

ENTRYPOINT ["/usr/local/bin/main"]
