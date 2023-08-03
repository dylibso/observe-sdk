FROM ubuntu

RUN apt update -y && apt install libssl-dev pkg-config openssl curl build-essential -y

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup toolchain add stable

WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .

CMD ["iota"]

