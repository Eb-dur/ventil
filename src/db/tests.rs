// Definitions of db-tests should be put here

#[cfg(test)]
mod tests {
    use crate::db::database::{get_connection, run};
    use crate::db::entities::{prelude::*, *};
    use crate::db::migrator;
    use async_std::task;
    use sea_orm::*;
    use sea_orm_migration::MigratorTrait;

    #[test]
    fn create_db_test(){
        create_db();
    }
    fn create_db() {

        let db = task::block_on(run());
        assert!(db.is_ok());
        let db = db.unwrap();
        let res = task::block_on(migrator::Migrator::up(&db, None));
        assert!(res.is_ok());
    }

    #[test]
    fn insert_owner_test(){
        insert_owner();   
    }
    fn insert_owner() {
        create_db();

        let db = task::block_on(get_connection());

        assert!(db.is_ok());

        let db = db.unwrap();

        let user_test = owner::ActiveModel {
            ..Default::default()
        };

        let res = task::block_on(Owner::insert(user_test).exec(&db));

        assert!(res.is_ok());
    }

    #[test]
    fn insert_item_test(){
        insert_item();
    }
    fn insert_item() {
        create_db();

        let db = task::block_on(get_connection());

        assert!(db.is_ok());

        let db = db.unwrap();

        let item_test = item::ActiveModel {
            item_type: ActiveValue::set("Disco".to_owned()),
            ..Default::default()
        };

        let res = task::block_on(Item::insert(item_test).exec(&db));

        assert!(res.is_ok());
    }

    #[test]
    fn insert_possession_test(){
        insert_possession();
    }

    fn insert_possession() {
        create_db();
        insert_owner();
        insert_item();

        let db = task::block_on(get_connection());
        assert!(db.is_ok());

        let db = db.unwrap();

        let owner = task::block_on(Owner::find_by_id(1).one(&db))
            .unwrap()
            .unwrap();

        let item = task::block_on(Item::find_by_id(1).one(&db))
            .unwrap()
            .unwrap();

        let possession_test = possession::ActiveModel {
            item: ActiveValue::set(item.id),
            owner: ActiveValue::set(owner.id),
            ..Default::default()
        };

        let res = task::block_on(Possession::insert(possession_test).exec(&db));

        assert!(res.is_ok());
    }
}
