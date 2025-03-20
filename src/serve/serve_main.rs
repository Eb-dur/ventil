use crate::db::database::set_up_db;
use rocket::*;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::item::routes::{ItemApiDoc, ItemRoutes};
use super::owner::routes::{OwnerApiDoc, OwnerRoutes};
use super::possession::routes::{PossessionApiDoc, PossessionRoutes};
use super::trade::logic::get_trades_mutex;

#[get("/")]
async fn index() -> &'static str {
    "Hello, traders!"
}

// Combine all API docs
#[derive(OpenApi)]
#[openapi(
    paths(
        // You can list specific paths here if needed
    ),
    components(
        // You can list specific components here if needed
    ),
    tags(
        (name = "items", description = "Item management API"),
        (name = "owners", description = "Owner management API"),
        (name = "possessions", description = "Possession management API")
    )
)]
struct ApiDoc;

async fn rocket() -> Rocket<Build> {
    let database = match set_up_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };

    rocket::build()
        .manage(database)
        .manage(get_trades_mutex())
        .mount("/", routes![index])
        .mount_items()
        .mount_owners()
        .mount_possessions()
        .mount(
            "/",
            SwaggerUi::new("/docs/<_..>").url(
                "/docs/api.json",
                ApiDoc::openapi()
                    .merge_from(ItemApiDoc::openapi())
                    .merge_from(OwnerApiDoc::openapi())
                    .merge_from(PossessionApiDoc::openapi()),
            ),
        )
}

pub async fn start_server() {
    println!("Starting Ventil server...");
    match rocket().await.launch().await {
        Ok(_) => println!("Server shutdown successfully"),
        Err(e) => eprintln!("Server error: {}", e),
    }
}
