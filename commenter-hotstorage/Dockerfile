FROM rust:1.72.1-slim-bullseye AS build

WORKDIR /app

COPY ./commenter-hotstorage commenter-hotstorage/
COPY ./commenter-database commenter-database/
COPY ./protos protos/

WORKDIR /app/commenter-hotstorage

RUN apt-get update \
    && apt-get install -y cmake gcc g++ \
    && apt-get install -y protobuf-compiler \
    && apt-get install -y libpq-dev

RUN cargo build --release   

FROM rust:1.72.1-slim-bullseye AS deploy
WORKDIR /app

COPY --from=build /app/commenter-hotstorage/target/release/commenter-hotstorage .

RUN apt-get update \
    && apt-get install -y libpq-dev

CMD ["./commenter-hotstorage"]