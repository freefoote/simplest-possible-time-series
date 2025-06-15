use diesel::prelude::*;
use serde_json::json;

mod helpers;
mod models;
mod schema;

fn main() {
    let connection = &mut helpers::establish_connection();

    let example_contents = json!({
        "test": 10,
        "more": 20.03
    });

    let new_entry = models::NewTsData {
        series_name: "Test",
        contents: &example_contents,
    };

    let returned = diesel::insert_into(schema::tsdata::table)
        .values(&new_entry)
        .returning(models::TsData::as_returning())
        .get_result(connection)
        .expect("Error saving new entry.");

    print!("Returned: {:?}", returned);
}
