FROM rust:1.72.1-slim-bullseye AS build

WORKDIR /app

COPY ./commenter-api commenter-api/
COPY ./commenter-database commenter-database/
COPY ./protos protos/

WORKDIR /app/commenter-api

RUN apt-get update \
    && apt-get install -y protobuf-compiler \
    && apt-get install -y libpq-dev

RUN cargo build --release   

FROM rust:1.72.1-slim-bullseye AS deploy
WORKDIR /app

COPY --from=build /app/commenter-api/target/release/commenter-api .

RUN apt-get update \
    && apt-get install -y libpq-dev

ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000

CMD ["./commenter-api"]