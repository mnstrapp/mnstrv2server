FROM rust:latest

WORKDIR /app

COPY . .

ENV SQLX_OFFLINE=true

RUN cargo install --path .

EXPOSE 8000
ENTRYPOINT ["mnstrv2server"]