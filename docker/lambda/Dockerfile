FROM ghcr.io/cargo-lambda/cargo-lambda as builder
WORKDIR /workspace
COPY . .
RUN apt update \
    && apt install --no-install-recommends -y libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*
