FROM rust:1.71 AS builder
COPY Cargo.toml ./
COPY ./src ./src

RUN cargo build --release

FROM debian:bullseye

# Needed for openssl(hyper uses it)
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder ./target/release/fplus-http-server ./target/release/fplus-http-server
CMD ["/target/release/fplus-http-server"]
