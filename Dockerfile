FROM rust:1.75

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

WORKDIR /app
COPY ./ ./

RUN cargo build

CMD ["cargo", "run"]
