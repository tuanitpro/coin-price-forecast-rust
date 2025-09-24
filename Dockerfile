FROM rust:1.90 as builder
WORKDIR /usr/src/app
COPY . .

# Build statically linked binary
RUN apt-get update && apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/main /
CMD ["/main"]
