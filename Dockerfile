FROM rust:1.86-bullseye as builder
WORKDIR /usr/src/n-chan
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y sqlite3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/n-chan /usr/local/bin/n-chan
COPY --from=builder /usr/src/n-chan/static .
COPY --from=builder /usr/src/n-chan/reset-db.sh .
COPY --from=builder /usr/src/n-chan/init.sql .
COPY --from=builder /usr/src/n-chan/defaults.sql .
RUN ./reset-db.sh
CMD ["n-chan"]