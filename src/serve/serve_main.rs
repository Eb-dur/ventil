use sea_orm::{EntityTrait, DatabaseConnection};
use crate::db::entities::prelude::Item;
use crate::db::database::set_up_db;
use rocket::serde::{Serialize,json::Json};
use rocket::*;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ItemResponse {
    item_type: String,
    id: i32,  // Adjust the type if your ID is not i32
}


#[get("/")]
async fn index() -> &'static str {
    "Hello, traders!"
}

#[get("/items")]
async fn items(database: &State<DatabaseConnection>) -> Json<Vec<ItemResponse>>{
    let db = database as &DatabaseConnection;

    let item_names = Item::find()
        .all(db)
        .await
        .unwrap()
        .into_iter()
        .map(|i| ItemResponse {
            item_type: i.item_type,
            id: i.id
        })
        .collect::<Vec<ItemResponse>>();

    Json(item_names)

}

async fn rocket() -> Rocket<Build> {
    let database = match set_up_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };

    rocket::build()
    .manage(database)
    .mount("/", routes![index, items])
}



pub async fn start_server() {
    println!("Starting Ventil server...");
    match rocket().await.launch().await {
        Ok(_) => println!("Server shutdown successfully"),
        Err(e) => eprintln!("Server error: {}", e),
    }
}