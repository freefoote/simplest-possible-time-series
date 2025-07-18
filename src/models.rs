use crate::schema::tsdata;
use chrono::DateTime;
use chrono::Utc;
use diesel::prelude::*;
use serde_json;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = tsdata)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TsData {
    pub id: i32,
    pub inserted_time: DateTime<Utc>,
    pub data_time: DateTime<Utc>,
    pub series_name: String,
    pub contents: serde_json::Value,
}

#[derive(Insertable)]
#[diesel(table_name = tsdata)]
pub struct NewTsData<'a> {
    pub data_time: DateTime<Utc>,
    pub series_name: &'a str,
    pub contents: &'a serde_json::Value,
}
