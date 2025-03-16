use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement};

const DATABASE_URL: &str = "sqlite:./ventil.db?mode=rwc";
const DB_NAME: &str = "ventil_db";


pub async fn set_up_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(DATABASE_URL).await?;
    Ok(db)
}
