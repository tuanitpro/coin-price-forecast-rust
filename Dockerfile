FROM rust:alpine3.22 AS builder
RUN apk add --no-cache musl-dev gcc g++ make

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
