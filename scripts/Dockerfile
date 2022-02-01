FROM ubuntu:20.04
LABEL name="SubstrateHealthNetwork"
LABEL maintainer="tirtad@dcentric.health"

ENV TZONE=Asia/Nicosia
ENV LANG=C.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL=C.UTF-8
RUN ln -snf /usr/share/zoneinfo/$TZONE /etc/localtime && echo $TZONE > /etc/timezone

RUN mkdir -p /root
COPY . /root

# Install build dependencies
RUN apt-get update -y \
    && apt-get install -y automake \
    build-essential \
    apt-utils \
    curl \
    && apt-get clean

RUN mkdir -p /root/setup
WORKDIR /root/setup
RUN curl https://getsubstrate.io -sSf | bash -s -- --fast
ENV PATH=/root/.cargo/bin:$PATH
RUN cargo --version
RUN rustup update nightly & \
    rustup update stable &\
    rustup target add wasm32-unknown-unknown --toolchain nightly

WORKDIR /root
RUN cargo clean
RUN cargo build --release

WORKDIR /root/target/release

# Expose ports
EXPOSE 9944
