FROM alpine:3.10

RUN cd && \
apk update && \
apk upgrade && \
apk add curl build-base

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal

WORKDIR /app/

COPY ./ /app/

RUN sh -c "source $HOME/.cargo/env && cargo build --release"

EXPOSE 80

CMD ["/app/target/release/mandelatar"]