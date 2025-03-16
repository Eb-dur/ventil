use crate::db::database::set_up_db;
use rocket::*;

use super::{item::routes::*, owner::routes::OwnerRoutes, possession::routes::PossessionRoutes};



#[get("/")]
async fn index() -> &'static str {
    "Hello, traders!"
}



async fn rocket() -> Rocket<Build> {
    let database = match set_up_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };

    rocket::build()
    .manage(database)
    .mount("/", routes![index])
    .mount_items()
    .mount_owners()
    .mount_possessions()
}



pub async fn start_server() {
    println!("Starting Ventil server...");
    match rocket().await.launch().await {
        Ok(_) => println!("Server shutdown successfully"),
        Err(e) => eprintln!("Server error: {}", e),
    }
}