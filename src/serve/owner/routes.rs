use crate::db::entities::{owner, prelude::Owner};
use rocket::{
    Build, Rocket, State, delete, get,
    http::Status,
    post,
    response::status::{Created, NotFound},
    routes,
    serde::{Deserialize, Serialize, json::Json},
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait};

pub trait OwnerRoutes {
    fn mount_owners(self) -> Self;
}

impl OwnerRoutes for Rocket<Build> {
    fn mount_owners(self) -> Self {
        self.mount(
            "/owners",
            routes![get_all_owners, get_owner_by_id, create_owner, delete_owner],
        )
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct OwnerResponse {
    pub id: i32,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateOwnerRequest {
    // Empty since Owner doesn't have any fields other than ID
    // which is auto-generated
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ApiResponse {
    pub message: String,
}

// GET /owners - Get all owners
#[get("/")]
pub async fn get_all_owners(database: &State<DatabaseConnection>) -> Json<Vec<OwnerResponse>> {
    let db = database as &DatabaseConnection;

    let owners = Owner::find()
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|o| OwnerResponse { id: o.id })
        .collect::<Vec<OwnerResponse>>();

    Json(owners)
}

// GET /owners/<id> - Get owner by ID
#[get("/<id>")]
pub async fn get_owner_by_id(
    id: i32,
    database: &State<DatabaseConnection>,
) -> Result<Json<OwnerResponse>, NotFound<Json<ApiResponse>>> {
    let db = database as &DatabaseConnection;

    match Owner::find_by_id(id).one(db).await {
        Ok(Some(owner)) => Ok(Json(OwnerResponse { id: owner.id })),
        _ => Err(NotFound(Json(ApiResponse {
            message: format!("Owner with id {} not found", id),
        }))),
    }
}

// POST /owners - Create a new owner
#[post("/", data = "<_owner_data>")]
pub async fn create_owner(
    _owner_data: Json<CreateOwnerRequest>,
    database: &State<DatabaseConnection>,
) -> Created<Json<OwnerResponse>> {
    let db = database as &DatabaseConnection;

    // Create active model
    let new_owner = owner::ActiveModel {
        ..Default::default()
    };

    // Insert and get the created owner
    let insert_result = new_owner.insert(db).await.unwrap();

    // Return with 201 Created status
    Created::new("/").body(Json(OwnerResponse {
        id: insert_result.id,
    }))
}

// DELETE /owners/<id> - Delete an owner
#[delete("/<id>")]
pub async fn delete_owner(
    id: i32,
    database: &State<DatabaseConnection>,
) -> Result<Status, NotFound<Json<ApiResponse>>> {
    let db = database as &DatabaseConnection;

    // Find the owner to delete
    let owner_result = Owner::find_by_id(id).one(db).await;

    match owner_result {
        Ok(Some(owner)) => {
            // Delete the owner
            owner.delete(db).await.unwrap();
            Ok(Status::NoContent)
        }
        _ => Err(NotFound(Json(ApiResponse {
            message: format!("Owner with id {} not found", id),
        }))),
    }
}
