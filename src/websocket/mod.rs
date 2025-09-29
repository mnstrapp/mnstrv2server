use rocket::Route;

pub mod battle_queue;
pub mod helpers;

pub fn routes() -> Vec<Route> {
    routes![battle_queue::handlers::battle_queue]
}
