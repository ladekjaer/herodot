FROM rust:latest

WORKDIR /usr/src/herodot
COPY . .

RUN cargo install --path .

EXPOSE 8080

CMD ["herodot"]
