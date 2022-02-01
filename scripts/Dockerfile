FROM ubuntu:20.04 as builder

RUN mkdir -p /app
WORKDIR /app

COPY . /app

RUN apt-get update -y \
    && apt-get install -y automake \
    build-essential \
    apt-utils \
    curl \
    && apt-get clean

RUN mkdir -p /app/setup
WORKDIR /app/setup

RUN curl https://getsubstrate.io -sSf | bash -s -- --fast
ENV PATH=/app/.cargo/bin:$PATH
RUN cargo --version
RUN rustup update nightly & \
    rustup update stable &\
    rustup target add wasm32-unknown-unknown --toolchain nightly

WORKDIR /app
RUN cargo clean
RUN cargo build --release

# =============

FROM phusion/baseimage:bionic-1.0.0
LABEL maintainer="tirtad@dcentric.health"

RUN useradd -m -u 1000 -U -s /bin/sh -d /aria aria

COPY --from=builder /app/target/release/aria /usr/local/bin

# checks
RUN ldd /usr/local/bin/aria && \
    /usr/local/bin/aria --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
    rm -rf /usr/bin /usr/sbin /usr/share/man

USER aria
EXPOSE 30333 9933 9944

RUN mkdir /aria/data

VOLUME ["/aria/data"]

ENTRYPOINT ["/usr/local/bin/aria"]
