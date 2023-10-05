FROM rust:1.71 AS builder
COPY Cargo.toml Cargo.lock gh-private-key.pem ./
COPY ./http-server ./http-server
COPY ./database ./database
COPY ./lib ./lib
COPY ./fplus ./fplus

# Change workdir to http-server and then build
WORKDIR ./http-server
RUN cargo build --release

FROM debian:bullseye

# Needed for openssl(hyper uses it)
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder ./http-server/target/release/http-server ./target/release/http-server
CMD ["/target/release/http-server"]
