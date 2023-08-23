FROM ubuntu

RUN apt update -y && apt install libssl-dev pkg-config openssl curl build-essential protobuf-compiler -y

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup toolchain add stable

ADD /rust /app/rust
ADD /proto /app/proto
ADD /demo-iota/rust /app/demo-iota/rust

WORKDIR /app/demo-iota/rust
RUN cargo install --path .

CMD ["iota"]

