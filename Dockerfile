# Build stage
FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM alpine:latest
WORKDIR /app
COPY --from=builder /app/target/release/main .
CMD ["./main"]