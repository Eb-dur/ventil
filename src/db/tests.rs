// Definitions of db-tests should be put here

#[cfg(test)]
mod tests {
    use crate::db::database::set_up_db;
    use crate::db::entities::{prelude::*, *};
    use crate::db::migrator;
    use sea_orm_migration::MigratorTrait;
    use sea_orm::*;

    #[tokio::test]
    async fn create_db_test(){
       create_db().await;
    }
    async fn create_db() {

        let db = set_up_db().await;
        assert!(db.is_ok());
        let db = db.unwrap();
        let res = migrator::Migrator::up(&db, None).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn insert_owner_test(){
        insert_owner().await
    }

    async fn insert_owner() {
        create_db().await;

        let db = set_up_db().await;

        assert!(db.is_ok());

        let db = db.unwrap();

        let user_test = owner::ActiveModel {
            ..Default::default()
        };

        let res = Owner::insert(user_test).exec(&db).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn insert_item_test(){
        insert_item().await
    }
    async fn insert_item() {
        create_db().await;

        let db = set_up_db().await;

        assert!(db.is_ok());

        let db = db.unwrap();

        let item_test = item::ActiveModel {
            item_type: ActiveValue::set("Disco".to_owned()),
            ..Default::default()
        };

        let res = Item::insert(item_test).exec(&db).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn insert_possession_test(){
        insert_possession().await;
    }

    async fn insert_possession() {
        create_db().await;
        insert_owner().await;
        insert_item().await;

        let db = set_up_db().await;
        assert!(db.is_ok());

        let db = db.unwrap();

        let owner = Owner::find_by_id(1).one(&db).await
            .unwrap()
            .unwrap();

        let item = Item::find_by_id(1).one(&db).await
            .unwrap()
            .unwrap();

        let possession_test = possession::ActiveModel {
            item: ActiveValue::set(item.id),
            owner: ActiveValue::set(owner.id),
            ..Default::default()
        };

        let res = Possession::insert(possession_test).exec(&db).await;

        assert!(res.is_ok());
    }
}
