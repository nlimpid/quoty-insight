FROM rust:1.85 as build


RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/debian.sources
RUN rustup target add x86_64-unknown-linux-musl && \
    apt update && \
    apt install -y pkg-config musl-tools musl-dev libssl-dev protobuf-compiler && \
    update-ca-certificates


COPY ./src ./src
COPY ./config.yml ./config.yml
COPY ./Cargo.toml .
COPY ./migration ./migration

RUN cargo build --release


FROM rust:1.85
RUN apt update && \
    apt install -y pkg-config musl-tools musl-dev libssl-dev protobuf-compiler && \
    update-ca-certificates

COPY --from=build ./target/release/quoty-insight /app/quoty-insight
COPY ./config.yml /app/config.yml
ENTRYPOINT ["./app/quoty-insight --config ./config.yml"]