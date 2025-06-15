use chrono::{DateTime, Local, TimeDelta};
use diesel::prelude::*;
use rand::{Rng, RngCore};
use serde_json::json;

mod helpers;
mod models;
mod schema;

fn delete_existing_series(connection: &mut PgConnection, name: &str) {
    use self::schema::tsdata::dsl::*;
    let num_deleted = diesel::delete(tsdata.filter(series_name.eq(&name)))
        .execute(connection)
        .expect("Error clearing series from database.");

    println!("Removed {} old records for series {}", num_deleted, &name);
}

fn generate_random_looking_git_commit_hash() -> String {
    let mut bytes = [0; 8];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(&bytes)
}

fn series_commit_code_sizes(connection: &mut PgConnection, number_of_entries: u32) {
    let series_name = "code_binary_sizes";

    // Delete existing series.
    delete_existing_series(connection, series_name);

    // And recreate it.
    let mut working_time: DateTime<Local> = Local::now();
    let mut rng = rand::rng();
    let mut loc = 1000;
    let mut bsize = 5_000_000;
    let mut test_coverage = 30.0;
    for quantity in 0..number_of_entries {
        // Adjust the values in the random range.
        loc += rng.random_range(-100..200);
        bsize += rng.random_range(-20_000..40_000);
        test_coverage += rng.random_range(-1.0..2.0);

        let example_contents = json!({
            "commit": generate_random_looking_git_commit_hash(),
            "lines_of_code": loc,
            "binary_size_bytes": bsize,
            "test_coverage_percent": test_coverage,
            "date": working_time.to_rfc3339()
        });

        // Advance time backwards a little bit (yes, we insert in reverse here)
        working_time = working_time
            .checked_sub_signed(TimeDelta::hours(1))
            .expect("Failed to adjust timestamp.");

        let new_entry = models::NewTsData {
            series_name: series_name,
            contents: &example_contents,
        };

        // TODO: Do a bulk insert of some kind for performance?
        diesel::insert_into(schema::tsdata::table)
            .values(&new_entry)
            .execute(connection)
            .expect("Error saving new entry.");

        // Calculate progress.
        if quantity % 500 == 0 {
            println!("Inserted {} rows for {}...", quantity + 1, series_name);
        }
    }

    println!(
        "Completed inserting {} rows for {}.",
        number_of_entries, series_name
    );
}

fn main() {
    let connection = &mut helpers::establish_connection();

    series_commit_code_sizes(connection, 5000);
}
