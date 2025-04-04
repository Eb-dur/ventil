//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.7

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "owner")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::possession::Entity")]
    Possession,
}

impl Related<super::possession::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Possession.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
