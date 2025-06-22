#[macro_use]
extern crate rocket;

use std::env;

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
Test With:
curl -X POST -H "Content-Type: application/json" -d '{
  "series": "abc-123",
  "data": {
    "name": "Custom Widget",
    "version": "1.0",
    "settings": {
      "color": "blue",
      "size": "medium"
    }
  }
}' http://localhost:8000/insert
*/

// Thanks to https://stackoverflow.com/questions/68633531/imlementing-connection-pooling-in-a-rust-diesel-app-with-r2d2 for a working solution.
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct ServerState {
    pub db_pool: DbPool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct SeriesInsertData {
    series: String,
    #[serde(rename = "data")]
    data: Value,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct InsertResponse {
    status: String,
    id: u64,
}

#[post("/insert", data = "<body>")]
async fn insert_data(
    state: &State<ServerState>,
    body: Json<SeriesInsertData>,
) -> Json<InsertResponse> {
    debug!("Received JSON: {:?}", body);

    let new_entry = models::NewTsData {
        series_name: &body.series,
        contents: &body.data,
    };

    let mut pooled = state.db_pool.get().expect("Failed to get connection.");
    let result = pooled.transaction(|conn| {
        diesel::insert_into(schema::tsdata::table)
            .values(&new_entry)
            .execute(conn)
    });

    print!("Result: {:?}", result);

    let response = InsertResponse {
        status: "success".to_string(),
        id: 12345,
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
