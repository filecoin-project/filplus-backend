FROM rust:1.71 AS builder
COPY Cargo.toml Cargo.lock gh-private-key.pem ./
COPY ./src ./src
RUN cargo build --release

FROM debian:bullseye

# Needed for openssl(hyper uses it)
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder ./target/release/http-server ./target/release/http-server
CMD ["/target/release/http-server"]