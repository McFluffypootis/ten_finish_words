
FROM rust

WORKDIR /app

RUN apt update && apt install lld clangd -y

COPY . .

ENV SQLX_OFFLINE true
ENV APP_ENVIRONMENT production

RUN cargo build --release

ENTRYPOINT ["./target/release/ten_finish_words"]

