FROM rust:1.72.1-slim-bullseye AS build

WORKDIR /app

COPY ./commenter-edge commenter-edge/
COPY ./protos protos/

WORKDIR /app/commenter-edge

RUN apt-get update \
    && apt-get install -y cmake gcc g++ \
    && apt-get install -y protobuf-compiler \
    && apt-get install -y pkg-config \
    && apt-get install -y libssl-dev

RUN cargo build --release   

FROM rust:1.72.1-slim-bullseye AS deploy
WORKDIR /app

COPY --from=build /app/commenter-edge/target/release/commenter-edge .

RUN apt-get update \
    && apt-get install -y libpq-dev

ENV WARP_ADDRESS=0.0.0.0
ENV WARP_PORT=5060

CMD ["./commenter-edge"]