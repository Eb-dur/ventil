use std::collections::HashMap;
use sea_orm_migration::prelude::*;
use async_std::task;
use sea_orm::DbErr;

use crate::migrator;
use crate::backend;

pub fn get_commads() -> HashMap< &'static str, fn()> {
    return HashMap::from([
        ("--help", help as fn()),
        ("-h", help as fn()),
        ("--migrate", do_migrate as fn()),
    ]);
}
fn do_migrate() {
    async fn run() -> Result<(), DbErr> {
        let db = backend::run().await.map_err(|e| {
            eprintln!("Error: Could not connect to the database. Reason: {:?}", e);
            e
        })?;

        migrator::Migrator::up(&db, None).await.map_err(|e| {
            eprintln!("Error: Could not refresh the database. Reason: {:?}", e);
            e
        })?;

        Ok(())
    }
    match task::block_on(run()) {
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
