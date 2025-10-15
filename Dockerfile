FROM lukemathwalker/cargo-chef:latest-rust-1.90.0 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# builder stage
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin ten_finish_words


FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Open SSL and TLS certifications + cleanup
RUN apt-get update -y \
	&& apt-get install -y --no-install-recommends openssl ca-certificates \
	&& apt-get autoremove -y \
	&& rm -rf /var/lib/lists/*

COPY --from=builder  app/target/release/ten_finish_words ten_finish_words
COPY configuration configuration
ENV APP_ENVIRONMENT production

ENTRYPOINT ["./ten_finish_words"]

