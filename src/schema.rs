// @generated automatically by Diesel CLI.

diesel::table! {
    tsdata (id) {
        id -> Int4,
        inserted_time -> Timestamptz,
        series_name -> Text,
        contents -> Jsonb,
    }
}
