#[macro_use]
extern crate rocket;

use std::env;

use chrono::{DateTime, Utc};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use diesel::{prelude::*, r2d2};
use dotenvy::dotenv;
use rocket::State;
use rocket::serde::{Deserialize, Serialize, json::Json};
use serde_json::Value;

mod models;
mod schema;

/*
Test With (date is optional - will default to current time if not provided):

With date:
curl -X POST -H "Content-Type: application/json" -d '{
  "series": "abc_123",
  "date": "2024-01-15T10:30:00Z",
  "data": {
    "commit": "12345678",
    "lines_of_code": 100,
    "binary_size_bytes": 100,
    "test_coverage_percent": 45.12
  }
}' http://localhost:8000/insert

Without date (uses current time):
curl -X POST -H "Content-Type: application/json" -d '{
  "series": "ABC_123",
  "data": {
    "commit": "12345678",
    "lines_of_code": 100,
    "binary_size_bytes": 100,
    "test_coverage_percent": 45.12
  }
}' http://localhost:8000/insert

Supported date formats:
- ISO 8601/RFC3339: "2024-01-15T10:30:00Z"
- Simple datetime: "2024-01-15 10:30:00"
- Date only: "2024-01-15" (defaults to midnight UTC)
*/

// Thanks to https://stackoverflow.com/questions/68633531/imlementing-connection-pooling-in-a-rust-diesel-app-with-r2d2 for a working solution.
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct ServerState {
    pub db_pool: DbPool,
}

// Custom date serializer/deserializer to handle multiple formats
mod date_format {
    use chrono::{DateTime, Utc, NaiveDateTime};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.to_rfc3339();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // Try parsing as RFC3339 first (ISO 8601)
        if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
            return Ok(dt.with_timezone(&Utc));
        }

        // Try parsing as naive datetime and assume UTC
        if let Ok(naive) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S") {
            return Ok(DateTime::from_naive_utc_and_offset(naive, Utc));
        }

        // Try parsing date only and set time to midnight UTC
        if let Ok(date) = chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
            let naive = date.and_hms_opt(0, 0, 0).unwrap();
            return Ok(DateTime::from_naive_utc_and_offset(naive, Utc));
        }

        Err(serde::de::Error::custom("Invalid date format"))
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct SeriesInsertData {
    series: String,
    #[serde(rename = "data")]
    data: Value,
    // Optional date field that defaults to current time if not provided
    #[serde(default = "chrono::Utc::now")]
    #[serde(with = "date_format")]
    date: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct InsertResponse {
    status: String,
    id: i32,
}

#[post("/insert", data = "<body>")]
async fn insert_data(
    state: &State<ServerState>,
    body: Json<SeriesInsertData>,
) -> Json<InsertResponse> {
    debug!("Received JSON: {:?}", body);

    let new_entry = models::NewTsData {
        data_time: body.date,
        series_name: &body.series,
        contents: &body.data,
    };

    let mut pooled = state.db_pool.get().expect("Failed to get connection.");
    let result = pooled
        .transaction(|conn| {
            diesel::insert_into(schema::tsdata::table)
                .values(&new_entry)
                .get_result::<models::TsData>(conn)
        })
        .unwrap();

    let response = InsertResponse {
        status: "success".to_string(),
        id: result.id,
    };

    Json(response)
}

pub fn establish_pooled_connection() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    return pool;
}

#[launch]
fn rocket() -> _ {
    // Load environment variables from .env file
    dotenv().ok();

    let db_pool: DbPool = establish_pooled_connection();

    rocket::build()
        .manage(ServerState { db_pool })
        .mount("/", routes![insert_data])
}
