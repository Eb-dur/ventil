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
use utoipa::{ToSchema, OpenApi};

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

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct OwnerResponse {
    pub id: i32,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct CreateOwnerRequest {
    // Empty since Owner doesn't have any fields other than ID
    // which is auto-generated
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct ApiResponse {
    pub message: String,
}

/// Get all owners
#[utoipa::path(
    get,
    path = "/owners",
    tags = ["owners"],
    responses(
        (status = 200, description = "List all owners successfully", body = [OwnerResponse])
    )
)]
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

/// Get owner by ID
#[utoipa::path(
    get,
    path = "/owners/{id}",
    tags = ["owners"],
    params(
        ("id" = i32, Path, description = "Owner identifier")
    ),
    responses(
        (status = 200, description = "Owner found successfully", body = OwnerResponse),
        (status = 404, description = "Owner not found", body = ApiResponse)
    )
)]
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

/// Create a new owner
#[utoipa::path(
    post,
    path = "/owners",
    tags = ["owners"],
    request_body = CreateOwnerRequest,
    responses(
        (status = 201, description = "Owner created successfully", body = OwnerResponse)
    )
)]
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

/// Delete an owner
#[utoipa::path(
    delete,
    path = "/owners/{id}",
    tags = ["owners"],
    params(
        ("id" = i32, Path, description = "Owner identifier")
    ),
    responses(
        (status = 204, description = "Owner deleted successfully"),
        (status = 404, description = "Owner not found", body = ApiResponse)
    )
)]
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

// Create the OpenAPI documentation using the utoipa macro
#[derive(OpenApi)]
#[openapi(
    paths(
        get_all_owners,
        get_owner_by_id,
        create_owner,
        delete_owner
    ),
    components(
        schemas(OwnerResponse, CreateOwnerRequest, ApiResponse)
    ),
    tags(
        (name = "owners", description = "Owner management API")
    )
)]
pub struct OwnerApiDoc;
