use sea_orm_migration::prelude::*;

use super::{m_20250314_000001_create_user_table::User, m_20250314_000002_create_item_table::Item};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20250315_000001_create_possesion_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Possession::Table)
                    .col(
                        ColumnDef::new(Possession::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Possession::User).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("Possession-owner")
                            .from(Possession::Table, Possession::User)
                            .to(User::Table, User::Id),
                    )
                    .col(ColumnDef::new(Possession::Item).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("Possesion-item")
                            .from(Possession::Table, Possession::Item)
                            .to(Item::Table, Item::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Possession::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Possession {
    Table,
    Id,
    Item,
    User,
}
