FROM rust as builder
WORKDIR app

RUN rustup default nightly-2021-06-17 && \
	rustup target add wasm32-unknown-unknown --toolchain nightly-2021-06-17

RUN apt-get update && \
	apt-get dist-upgrade -y -o Dpkg::Options::="--force-confold" && \
	apt-get install -y cmake pkg-config libssl-dev git clang libclang-dev

ARG GIT_COMMIT=
ENV GIT_COMMIT=$GIT_COMMIT
ARG BUILD_ARGS

COPY . .

RUN cargo build --release $BUILD_ARGS

# =============

FROM phusion/baseimage:bionic-1.0.0
LABEL name="SubstrateHealthNetwork"
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
