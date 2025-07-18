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

    // Process in batches for memory efficiency
    const BATCH_SIZE: usize = 500;
    let mut total_inserted = 0;

    while total_inserted < number_of_entries {
        let batch_size = std::cmp::min(BATCH_SIZE, (number_of_entries - total_inserted) as usize);
        let mut data_points = Vec::with_capacity(batch_size);

        // Collect data for this batch
        for _ in 0..batch_size {
            // Adjust the values in the random range.
            loc += rng.random_range(-100..200);
            bsize += rng.random_range(-20_000..40_000);
            test_coverage += rng.random_range(-1.0..2.0);

            let example_contents = json!({
                "commit": generate_random_looking_git_commit_hash(),
                "lines_of_code": loc,
                "binary_size_bytes": bsize,
                "test_coverage_percent": test_coverage,
            });

            // Advance time backwards a little bit (yes, we insert in reverse here)
            working_time = working_time
                .checked_sub_signed(TimeDelta::hours(1))
                .expect("Failed to adjust timestamp.");

            // Store the data for this batch
            data_points.push((working_time.into(), example_contents));
        }

        // Create the entries vector with references to the stored data
        let entries: Vec<models::NewTsData> = data_points
            .iter()
            .map(|(data_time, contents)| models::NewTsData {
                data_time: *data_time,
                series_name: series_name,
                contents: contents,
            })
            .collect();

        // Perform bulk insert for this batch
        diesel::insert_into(schema::tsdata::table)
            .values(&entries)
            .execute(connection)
            .expect("Error saving batch entries.");

        total_inserted += batch_size as u32;
        println!("Inserted {} of {} rows for {}...", total_inserted, number_of_entries, series_name);
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
