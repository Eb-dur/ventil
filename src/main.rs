use std::env;
use std::collections::HashMap;

mod backend;
mod migrator;
mod commands;

fn main() {
    let commands: HashMap<&str, fn()> = commands::get_commads();

    let args: Vec<String> = env::args().collect();
    
    for command in &args[1..]{
        match commands.get(&command.as_str()) {
            Some(func) => func(),
            None => eprintln!("Error: {} is not a command, --help for list of commands", &command),
        }
    }

}
