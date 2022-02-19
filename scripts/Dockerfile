FROM rust:1.50 as builder

ENV TZONE=Asia/Nicosia
ENV LANG=C.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL=C.UTF-8
RUN ln -snf /usr/share/zoneinfo/$TZONE /etc/localtime && echo $TZONE > /etc/timezone

RUN mkdir -p /root
WORKDIR /root

COPY . /root

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

# =============

FROM debian:buster-slim

LABEL name="MetaNetwork"
LABEL maintainer="tirtad@dcentric.health"

ENV TZONE=Asia/Nicosia
ENV LANG=C.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL=C.UTF-8
RUN ln -snf /usr/share/zoneinfo/$TZONE /etc/localtime && echo $TZONE > /etc/timezone

COPY --from=builder /root/target/release/meta /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /meta meta

# checks
RUN ldd /usr/local/bin/meta
RUN /usr/local/bin/meta --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
    rm -rf /usr/bin /usr/sbin /usr/share/man

USER meta
EXPOSE 30333 9933 9944

RUN mkdir /meta/data

VOLUME ["/meta/data"]

ENTRYPOINT ["/usr/local/bin/meta"]
