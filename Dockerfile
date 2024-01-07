FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main

ARG DEBIAN_FRONTEND=noninteractive

ENV TZ=Europe/Berlin5
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt-get update
RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt install -y pkg-config:arm64 libdbus-1-dev:arm64 libsqlite3-dev:arm64