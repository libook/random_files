# Use Rust official image as builder, on the same Debian release as the runtime stage
FROM rust:1-slim-trixie as builder
WORKDIR /

# Copy Cargo files and compile dependencies
COPY . ./
RUN cargo build --release

FROM debian:trixie-slim
RUN apt-get update && apt-get install --no-install-recommends -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# Copy binary from builder stage
COPY --from=builder /target/release/random_files /random_files
EXPOSE 3030/tcp
# Run the application
CMD ["/random_files"]
