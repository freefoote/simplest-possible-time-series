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
use rocket::{State, http::Status, Request, catch, catchers, response::Responder, Response};
use rocket::serde::{Deserialize, Serialize, json::Json};
use serde_json::Value;
use std::io::Cursor;

mod models;
mod schema;

/*
Test With (date is optional - will default to current time if not provided):

Successful request:
curl -X POST -H "Content-Type: application/json" -d '{
  "series": "abc_123",
  "date": "2024-01-15T10:30:00Z",
  "data": {"test": "data"}
}' http://localhost:8000/insert

Test constraint violation (returns actual database error message):
curl -X POST -H "Content-Type: application/json" -d '{
  "series": "invalid-format",
  "data": {"test": "data"}
}' http://localhost:8000/insert

Test 404 (returns JSON error):
curl -X POST -H "Content-Type: application/json" -d '{}' http://localhost:8000/nonexistent

Supported date formats:
- ISO 8601/RFC3339: "2024-01-15T10:30:00Z"
- Simple datetime: "2024-01-15 10:30:00"
- Date only: "2024-01-15" (defaults to midnight UTC)

All errors now return JSON with actual error messages:
Success: {"status": "success", "id": 123}
Database constraint: {"status": "error", "error": "new row for relation \"tsdata\" violates check constraint \"series_format_check\""}
404: {"status": "error", "error": "Not found"}
*/

// Thanks to https://stackoverflow.com/questions/68633531/imlementing-connection-pooling-in-a-rust-diesel-app-with-r2d2 for a working solution.
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct ServerState {
    pub db_pool: DbPool,
}

// Custom date serializer/deserializer to handle multiple formats
// (Courtesy of LLM, Claude Sonnet 4.)
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct ErrorResponse {
    status: String,
    error: String,
}

// Custom error type that carries the actual error message
// (Courtesy of LLM, Claude Sonnet 4.)
#[derive(Debug)]
struct ApiError {
    status: Status,
    message: String,
}

impl ApiError {
    fn new(status: Status, message: String) -> Self {
        Self { status, message }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let error_response = ErrorResponse {
            status: "error".to_string(),
            error: self.message,
        };

        let json_string = serde_json::to_string(&error_response).unwrap();

        Response::build()
            .status(self.status)
            .header(rocket::http::ContentType::JSON)
            .sized_body(json_string.len(), Cursor::new(json_string))
            .ok()
    }
}

#[catch(default)]
fn default_catcher(status: Status, _req: &Request) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        status: "error".to_string(),
        error: format!("Error {}: {}", status.code, status.reason().unwrap_or("Unknown error")),
    })
}

#[post("/insert", data = "<body>")]
async fn insert_data(
    state: &State<ServerState>,
    body: Json<SeriesInsertData>,
) -> Result<Json<InsertResponse>, ApiError> {
    debug!("Received JSON: {:?}", body);

    let new_entry = models::NewTsData {
        data_time: body.date,
        series_name: &body.series,
        contents: &body.data,
    };

    let mut pooled = state.db_pool.get()
        .map_err(|e| ApiError::new(Status::InternalServerError, format!("Database connection failed: {}", e)))?;

    let result = pooled.transaction(|conn| {
        diesel::insert_into(schema::tsdata::table)
            .values(&new_entry)
            .get_result::<models::TsData>(conn)
    }).map_err(|e| {
        // Extract the actual database error message
        let error_msg = match e {
            diesel::result::Error::DatabaseError(_, info) => {
                info.message().to_string()
            }
            _ => format!("Database error: {}", e)
        };
        ApiError::new(Status::BadRequest, error_msg)
    })?;

    let response = InsertResponse {
        status: "success".to_string(),
        id: result.id,
    };

    Ok(Json(response))
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
        .register("/", catchers![default_catcher])
}
