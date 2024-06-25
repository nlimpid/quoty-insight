FROM rust:1.79.0-slim as build

RUN rustup target add x86_64-unknown-linux-musl && \
    apt update && \
    apt install -y musl-tools musl-dev && \
    update-ca-certificates


COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .

RUN cargo build --target x86_64-unknown-linux-musl --release


FROM rust:1.79-alpine3.20
COPY --from=build ./target/x86_64-unknown-linux-musl/release/quoty-insight /app/quoty-insight

ENTRYPOINT ["./app/quoty-insight"]