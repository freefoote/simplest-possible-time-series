# Simples Possible Time Series System

## Setup local environment

We stand on the shoulders of giants, so you'll need:

* Rust - use standard `rustup` to install your toolchain.
* Diesel as an ORM layer - see below.
* docker-compose and Docker for running external services (Postgres and Grafana). Install as per your preferred method.

For Diesel, follow their [getting started guide](https://diesel.rs/guides/getting-started) to install the CLI tool. In this case, we just need Postgres, so you can use:

```bash
sudo apt-get install libpq-dev
cargo install diesel_cli --no-default-features --features postgres
```

For docker-compose, you can fire this up as usual:

```bash
docker compose up -d
```

Copy over the env file, which is configured for `docker-compose.yml`, but adjust for your environment:

```bash
cp .env.example .env
```

And then run the migrations to get your database set up:

```bash
diesel migration run
```

Finally, run the test-data-generator to insert some sample data to work with.