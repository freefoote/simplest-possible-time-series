CREATE TABLE tsdata (
  id SERIAL PRIMARY KEY,
  inserted_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  data_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  -- Text has been chosen here as it's equivalent in speed.
  -- However, Postgres will only index the first 2712 bytes of the field;
  -- in our use case, I would imagine that it would be shorter than this.
  series_name TEXT NOT NULL,
  contents JSONB NOT NULL
);

CREATE INDEX tsdata_data_time_index ON tsdata (data_time);
CREATE INDEX tsdata_series_name_index ON tsdata (series_name);

ALTER TABLE tsdata ADD CONSTRAINT series_name_format_check CHECK (series_name ~ '^[a-z0-9_]+$');