FROM rust:1.90 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/static-debian12
WORKDIR /bin

# Install runtime dependencies for Rust + OpenSSL
# RUN apt-get update && apt-get install -y \
#     ca-certificates \
#     libssl3 \
#     && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/main .
ENTRYPOINT ["main"]
