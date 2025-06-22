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

Finally, run the test-data-generator to insert some sample data to work with:

```bash
cargo run --bin test-data-generator
```

## Grafana

With docker-compose, Grafana is available via [http://localhost:3000/](http://localhost:3000/), default login is admin/admin.

You then need to add a Postgres Data source to connect to itself:

* Connections > Data sources > Add new Data Source
* Type: Postgres
* Credentials: fill in based on DATABASE_URL from .env, except the host URL is `postgres:5432` as we use the container DNS name.
* Be sure to set TLS/SSL mode to "disable" for the docker-compose setup.
* Be sure to set the Version to match your running version (13 in my test, because this matched my legacy deployment version)

The default dashboard uses a custom view I made to unpack the JSON data. This is a bit of a hacky workaround for now, as it should be possible to transform this directly with Grafana.

Connect via `psql` and apply the view:

```bash
psql postgres://test:test@localhost:5432/test < views/example_code_binary_sizes.sql
```

Now in Grafana, create a new dashboard, and add a new panel. Then go back to the dashboard, and for that panel, in the three dots menu, go to Inspect > Panel JSON. Then you can replace the JSON with `views/grafana-panel-code-binary-size.json` for an example. If it gives you an error, edit the panel and run the query the first time. Then it should work ok!
