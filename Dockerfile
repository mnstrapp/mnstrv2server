FROM rust:latest

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

ENV SQLX_OFFLINE=true

RUN cargo install --path .

EXPOSE 8080
EXPOSE 50051
ENTRYPOINT ["mnstrv2server"]