use std::env;
use std::collections::HashMap;

use async_std::task;
use sea_orm_migration::prelude::*;

mod backend;
mod migrator;

fn do_migrate(){
    async fn run() -> Result<(), DbErr>{
        let db = backend::run().await
            .map_err(|e| {
                eprintln!("Error: Could not connect to the database. Reason: {:?}", e);
                e
        })?;
        
        migrator::Migrator::up(&db, None).await.
            map_err(|e| {
                eprintln!("Error: Could not refresh the database. Reason: {:?}", e);
                e
        })?;
        
        Ok(())
    }
    match task::block_on(run()){
        Err(e) => eprintln!("Error: Could not migrate database!, \n Reason: {e}"),
        Ok(()) => println!("Success, database migrated!"),
    }
}

fn help() {
    fn print_command( command : &str, description : &str){
        println!("   {command}    {description}");
    }
    println!("--------Welcome to Ventil!--------");
    println!("Commands:");
    print_command("--help", "Display this menu");
    print_command("--migrate", "Apply migrations");
}

fn main() {
    let commands: HashMap<&str, fn()> = HashMap::from([
        ("--help", help as fn()),
        ("-h", help as fn()),
        ("--migrate", do_migrate as fn())
    ]);

    let args: Vec<String> = env::args().collect();
    
    for command in &args[1..]{
        match commands.get(&command.as_str()) {
            Some(func) => func(),
            None => eprintln!("Error: {} is not a command, --help for list of commands", &command),
        }
    }



}
