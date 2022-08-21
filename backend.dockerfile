FROM alpine:3.10 AS builder

RUN cd && \
apk update && \
apk upgrade && \
apk add curl build-base

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal

WORKDIR /app/

COPY ./mandelatar-core /app/mandelatar-core
COPY ./backend_rust /app/backend_rust

RUN sh -c "source $HOME/.cargo/env && cd backend_rust && cargo build --release"

FROM alpine:3.10

WORKDIR /app/

COPY --from=builder /app/backend_rust/target/release/mandelatar /app/mandelatar

EXPOSE 80

CMD ["/app/mandelatar"]