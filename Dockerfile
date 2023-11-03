from rust:bookworm as builder

RUN mkdir /app 
RUN mkdir /app/bin 

COPY src /app/src/
COPY Cargo.toml /app

RUN apt-get update && apt-get install -y libssl-dev pkg-config make
RUN cargo install --path /app --root /app
RUN strip app/bin/mongodb-rest-rs

FROM debian:bookworm-slim
RUN apt-get update && apt install -y openssl
WORKDIR /app
COPY --from=builder /app/bin/ ./

ENTRYPOINT ["/app/mongodb-rest-rs"]
EXPOSE 8080
