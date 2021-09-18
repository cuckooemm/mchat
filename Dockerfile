FROM rust:1.55 as builder

WORKDIR /workspace

COPY ./Cargo.lock ./Cargo.toml ./
COPY ./config $CARGO_HOME/
RUN mkdir -p src/server/
RUN echo "fn main() {println!(\"hello world!\")}" > src/server/main.rs
RUN cargo build --bin server --release
RUN rm -f target/release/deps/server*
RUN rm -rf src/*
COPY ./src ./src

RUN cargo build --bin server --release
RUN mv target/release/server .

FROM debian:stretch-slim AS app

WORKDIR /workspace

RUN mkdir bin/ && mkdir logs
COPY --from=builder /workspace/server ./bin/
# COPY --from=builder /workspace/target/x86_64-unknown-linux-musl/release/mchat ./bin/
COPY Config.toml Config.toml
EXPOSE 8000
CMD [ "./bin/server" ]
