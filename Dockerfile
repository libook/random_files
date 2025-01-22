# Use Rust official image as builder
FROM rustlang/rust:nightly-bullseye-slim as builder
WORKDIR /

RUN apt-get update
RUN apt-get -y install pkg-config libssl-dev
RUN rustup update

# Copy Cargo files and compile dependencies
COPY . ./
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install --no-install-recommends -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# Copy binary from builder stage
COPY --from=builder /target/release/random_files /random_files
EXPOSE 3030/tcp
# Run the application
CMD ["/random_files"]
