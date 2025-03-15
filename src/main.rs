use std::collections::HashMap;
use std::env;

mod commands;
mod db;

fn main() {
    let commands: HashMap<&str, fn()> = commands::get_commads();

    let args: Vec<String> = env::args().collect();

    for command in &args[1..] {
        match commands.get(&command.as_str()) {
            Some(func) => func(),
            None => eprintln!(
                "Error: {} is not a command, --help for list of commands",
                &command
            ),
        }
    }
}
