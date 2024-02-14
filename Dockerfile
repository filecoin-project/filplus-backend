FROM rust:1.72 AS builder
COPY ./fplus-lib /fplus-lib
COPY ./fplus-http-server/Cargo.toml /fplus-http-server/Cargo.toml
COPY ./fplus-http-server/src /fplus-http-server/src
COPY ./fplus-database/Cargo.toml /fplus-database/Cargo.toml
COPY ./fplus-database/src /fplus-database/src
WORKDIR /fplus-http-server
RUN cargo build --release

FROM debian:bookworm
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /fplus-http-server/target/release/fplus-http-server /target/release/fplus-http-server
CMD ["/target/release/fplus-http-server"]