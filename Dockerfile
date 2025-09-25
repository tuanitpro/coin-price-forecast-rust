FROM rust:alpine AS builder
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    && rm -rf /var/cache/apk/*

RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /app

COPY Cargo.lock Cargo.toml ./
RUN cargo fetch

RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release

RUN rm -rf src

COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
WORKDIR /bin

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/main .

ENTRYPOINT ["./main"]
