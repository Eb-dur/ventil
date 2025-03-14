use std::env;
use std::collections::HashMap;

use futures::executor::block_on;

mod backend;

fn help() {
    fn print_command( command : &str, description : &str){
        println!("   {command}    {description}");
    }
    println!("--------Welcome to Ventil!--------");
    println!("Commands:");
    print_command("--help", "Display this menu");
    
}

fn main() {
    let commands = HashMap::from([
        ("--help", help)
    ]);

    let args: Vec<String> = env::args().collect();
    
    for command in &args[1..]{
        match commands.get(&command.as_str()) {
            Some(func) => func(),
            None => eprintln!("Error: {} is not a command", &command),
        }
    }

    if let Err(err) = block_on(backend::run()) {
        panic!("{}", err);
    }

}
