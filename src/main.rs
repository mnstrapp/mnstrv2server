#[macro_use]
extern crate rocket;

use rocket_cors::CorsOptions;
use sqlx::postgres::PgPoolOptions;
use std::{env, net::SocketAddr};
use tonic::transport::Server as GrpcServer;
use tonic_reflection::server::Builder as GrpcReflectionBuilder;

use crate::proto::{mnstr_service_server::MnstrServiceServer, session_service_server::SessionServiceServer, user_service_server::UserServiceServer};

pub mod proto {
    tonic::include_proto!("mnstrv2");
}

mod database;
mod graphql;
mod models;
mod services;
mod utils;
mod websocket;
mod battle;

const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("descriptor");

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = env::var("TWILIO_ACCOUNT_SSID")?;
    let _ = env::var("TWILIO_AUTH_TOKEN")?;
    let _ = env::var("TWILIO_PHONE_NUMBER")?;
    let _ = env::var("SENDGRID_API_KEY")?;
    let _ = env::var("SENDGRID_FROM_EMAIL")?;
    let grpc_port = env::var("GRPC_PORT")?.parse::<u16>()?;
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new().connect(&*database_url).await?;
    let cors = CorsOptions::default().to_cors().unwrap();

    let session_service =
        SessionServiceServer::new(services::sessions::SessionServiceImpl::default());
    let mnstr_service = MnstrServiceServer::new(services::mnstrs::MnstrServiceImpl::default());
    let users_service = UserServiceServer::new(services::users::UserServiceImpl::default());
    let reflection_service = GrpcReflectionBuilder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()
        .expect("reflection service could not build");
   
    let _ = tokio::spawn(async move {
        GrpcServer::builder()
            .add_service(session_service)
            .add_service(mnstr_service)
            .add_service(users_service)
            .add_service(reflection_service)
            .serve(SocketAddr::from(([0, 0, 0, 0], grpc_port)))
            .await
    });

    rocket::build()
        .mount("/", routes![index])
        .mount("/graphql", graphql::routes())
        .mount("/ws", websocket::routes())
        .mount("/static", rocket::fs::FileServer::from("static"))
        .manage(pool)
        .attach(cors)
        .launch()
        .await?;
    Ok(())
}