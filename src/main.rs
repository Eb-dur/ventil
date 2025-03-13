use std::env;
use std::collections::HashMap;

fn hello_world() {
    println!("Hello world!");
}

fn main() {
    let commands = HashMap::from([
        ("--hello-world", hello_world)
    ]);

    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    
    for command in &args[1..]{
        match commands.get(&command.as_str()) {
            Some(func) => func(),
            None => eprintln!("Error: {} is not a command", &command),
        }
    }

}
