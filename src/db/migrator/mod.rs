use sea_orm_migration::prelude::*;

mod m_20250314_000001_create_owner_table;
mod m_20250314_000002_create_item_table;
mod m_20250315_000001_create_possesion_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m_20250314_000001_create_owner_table::Migration),
            Box::new(m_20250314_000002_create_item_table::Migration),
            Box::new(m_20250315_000001_create_possesion_table::Migration),
        ]
    }
}
