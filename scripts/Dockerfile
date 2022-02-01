# FROM ubuntu:20.04
# LABEL name="SubstrateHealthNetwork"
# LABEL maintainer="tirtad@dcentric.health"

# ENV TZONE=Asia/Nicosia
# ENV LANG=C.UTF-8
# ENV LANGUAGE en_US:en
# ENV LC_ALL=C.UTF-8
# RUN ln -snf /usr/share/zoneinfo/$TZONE /etc/localtime && echo $TZONE > /etc/timezone

# RUN mkdir -p /root
# COPY . /root

# # Install build dependencies
# RUN apt-get update -y \
#     && apt-get install -y automake \
#     build-essential \
#     apt-utils \
#     curl \
#     && apt-get clean

# RUN mkdir -p /root/setup
# WORKDIR /root/setup
# RUN curl https://getsubstrate.io -sSf | bash -s -- --fast
# ENV PATH=/root/.cargo/bin:$PATH
# RUN cargo --version
# RUN rustup update nightly & \
#     rustup update stable &\
#     rustup target add wasm32-unknown-unknown --toolchain nightly

# WORKDIR /root
# RUN cargo clean
# RUN cargo build --release

# WORKDIR /root/target/release

# # Expose ports
# EXPOSE 9944

FROM rust as builder
WORKDIR /app

RUN rustup default nightly-2021-06-17 && \
    rustup target add wasm32-unknown-unknown --toolchain nightly-2021-06-17

RUN apt-get update && \
    apt-get dist-upgrade -y -o Dpkg::Options::="--force-confold" && \
    apt-get install -y cmake pkg-config libssl-dev git clang libclang-dev

COPY . .

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
