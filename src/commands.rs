use sea_orm::DbErr;
use sea_orm_migration::prelude::*;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::db::{database, migrator};
use crate::serv::serv_main;



type AsyncFn = fn() -> Pin<Box<dyn Future<Output = ()> + Send>>;

pub fn get_commands() -> HashMap<&'static str, AsyncFn> {
    let help_fn: AsyncFn = || Box::pin(async { help() });
    let migrate_fn: AsyncFn = || Box::pin(async { do_migrate().await });
    let serve_fn: AsyncFn = || Box::pin(serv_main::start_server());
    
    return HashMap::from([
        ("--help", help_fn),
        ("-h", help_fn),
        ("--migrate", migrate_fn),
        ("--serve", serve_fn),
    ]);
}

async fn do_migrate() {
    async fn run() -> Result<(), DbErr> {
        let db = database::run().await.map_err(|e| {
            eprintln!("Error: Could not connect to the database. Reason: {:?}", e);
            e
        })?;

        migrator::Migrator::up(&db, None).await.map_err(|e| {
            eprintln!("Error: Could not refresh the database. Reason: {:?}", e);
            e
        })?;

        Ok(())
    }
    match run().await {
        Err(e) => eprintln!("Error: Could not migrate database!, \n Reason: {e}"),
        Ok(()) => println!("Success, database migrated!"),
    }
}

fn help() {
    fn print_command(command: &str, description: &str) {
        println!("   {command}    {description}");
    }
    println!("--------Welcome to Ventil!--------");
    println!("Commands:");
    print_command("--help", "Display this menu");
    print_command("--migrate", "Apply migrations");
}
