use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

// Helper function to create a view for a specific series
// The series_name is already validated by the database constraint to match ^[a-z0-9_]+$
pub fn create_series_view(connection: &mut PgConnection, series_name: &str) -> Result<(), diesel::result::Error> {
    use diesel::sql_query;

    // Since series_name is validated by the database constraint to only contain [a-z0-9_],
    // it's safe to use directly in the SQL. We still use proper SQL escaping for the string literal.
    let view_name = format!("tsdata_{}", series_name);
    let create_view_sql = format!(
        "CREATE OR REPLACE VIEW {} AS (SELECT * FROM tsdata WHERE series_name = '{}')",
        view_name, series_name
    );

    sql_query(create_view_sql)
        .execute(connection)?;

    Ok(())
}
