FROM rust:latest as builder

WORKDIR /usr/src/app

COPY . .

RUN cargo lambda build --release

FROM debian:buster-slim

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/your_binary_name .

CMD ["./rustetl"]