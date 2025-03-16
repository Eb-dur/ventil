use std::env;

mod commands;
mod db;
mod serv;

#[rocket::main]
async fn main() {
    let commands = commands::get_commands();

    let args: Vec<String> = env::args().collect();

    for command in &args[1..] {
        match commands.get(&command.as_str()) {
            Some(func) => func().await,
            None => eprintln!(
                "Error: {} is not a command, --help for list of commands",
                &command
            ),
        }
    }
}
