# 1. This tells docker to use the Rust official image
FROM rust

# Create app directory
WORKDIR /app

COPY ./Cargo.toml ./Cargo.toml
COPY ./src/lib.rs ./src/lib.rs

# Build your program for release
RUN cargo build --release

# 2. Copy the files in your machine to the Docker image
COPY ./ ./

# Build your program for release
RUN cargo build --release

EXPOSE 80
# Run the binary
CMD ["./target/release/rim"]