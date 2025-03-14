use std::env;
use std::collections::HashMap;

use async_std::task;
use sea_orm_migration::prelude::*;

mod backend;
mod migrator;

fn do_migrate(){
    println!("Here!");
    async fn run() -> Result<(), DbErr>{
        let db = backend::run().await.unwrap();
        let schema_manager = SchemaManager::new(&db);
        migrator::Migrator::refresh(&db).await?;
        assert!(schema_manager.has_table("user").await?);
        Ok(())
    }
    task::block_on(run());
}

fn help() {
    fn print_command( command : &str, description : &str){
        println!("   {command}    {description}");
    }
    println!("--------Welcome to Ventil!--------");
    println!("Commands:");
    print_command("--help", "Display this menu");
    
}

fn main() {
    let commands: HashMap<&str, fn()> = HashMap::from([
        ("--help", help as fn()),
        ("--migrate", do_migrate as fn())
    ]);

    let args: Vec<String> = env::args().collect();
    
    for command in &args[1..]{
        match commands.get(&command.as_str()) {
            Some(func) => func(),
            None => eprintln!("Error: {} is not a command", &command),
        }
    }



}
