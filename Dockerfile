FROM rust:1.71 AS builder
COPY ./fplus-lib /fplus-lib
COPY ./fplus-http-server/Cargo.toml /fplus-http-server/Cargo.toml
COPY ./fplus-http-server/src /fplus-http-server/src
WORKDIR /fplus-http-server
RUN cargo build --release

FROM debian:bullseye
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /fplus-http-server/target/release/fplus-http-server /target/release/fplus-http-server
CMD ["/target/release/fplus-http-server"]