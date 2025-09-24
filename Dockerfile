FROM rust:1.90 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM scratch
COPY --from=builder /usr/src/app/target/release/main /
CMD ["./main"]
