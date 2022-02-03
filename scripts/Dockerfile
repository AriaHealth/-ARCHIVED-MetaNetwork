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

LABEL name="AriaHealthNetwork"
LABEL maintainer="tirtad@dcentric.health"

ENV TZONE=Asia/Nicosia
ENV LANG=C.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL=C.UTF-8
RUN ln -snf /usr/share/zoneinfo/$TZONE /etc/localtime && echo $TZONE > /etc/timezone

COPY --from=builder /root/target/release /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /aria aria

# checks
RUN ldd /usr/local/bin/aria
RUN /usr/local/bin/aria --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
    rm -rf /usr/bin /usr/sbin /usr/share/man

USER aria
EXPOSE 30333 9933 9944

RUN mkdir /aria/data

VOLUME ["/aria/data"]

ENTRYPOINT ["/usr/local/bin/aria"]
