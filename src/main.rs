#[macro_use]
extern crate rocket;

use rocket_cors::CorsOptions;
use sqlx::postgres::PgPoolOptions;
use std::env;
mod database;
mod graphql;
mod models;
mod utils;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let _ = env::var("TWILIO_ACCOUNT_SSID")?;
    let _ = env::var("TWILIO_AUTH_TOKEN")?;
    let _ = env::var("TWILIO_PHONE_NUMBER")?;
    let _ = env::var("SENDGRID_API_KEY")?;
    let _ = env::var("SENDGRID_FROM_EMAIL")?;
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new().connect(&*database_url).await?;
    let cors = CorsOptions::default().to_cors().unwrap();

    rocket::build()
        .mount("/", routes![index])
        .mount("/graphql", graphql::routes())
        .manage(pool)
        .attach(cors)
        .launch()
        .await?;
    Ok(())
}
