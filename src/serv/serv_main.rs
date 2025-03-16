use rocket::*;


#[get("/")]
async fn index() -> &'static str {
    "Hello, traders!"
}

fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![index])
}

pub async fn start_server() {
    println!("Starting Ventil server...");
    match rocket().launch().await {
        Ok(_) => println!("Server shutdown successfully"),
        Err(e) => eprintln!("Server error: {}", e),
    }
}