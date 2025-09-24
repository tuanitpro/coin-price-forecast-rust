FROM rust:1.90 as builder
WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y musl-tools pkg-config libssl-dev
RUN rustup target add x86_64-unknown-linux-musl

COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/main /
CMD ["./main"]
