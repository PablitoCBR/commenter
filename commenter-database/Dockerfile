FROM rust:1.72.1-slim-bullseye

WORKDIR /app

COPY ./commenter-database .

RUN apt-get update \
    && apt-get install -y protobuf-compiler \
    && apt-get install -y libpq-dev \
    && cargo install diesel_cli --no-default-features --features postgres

CMD ["diesel", "migration", "run" ]