use crate::db::entities::{item, prelude::Item};
use rocket::{
    Build, Rocket, State, delete, get,
    http::Status,
    post, put,
    response::status::{Created, NotFound},
    routes,
    serde::{Deserialize, Serialize, json::Json},
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, ModelTrait};
use utoipa::{ToSchema, OpenApi};

pub trait ItemRoutes {
    fn mount_items(self) -> Self;
}

impl ItemRoutes for Rocket<Build> {
    fn mount_items(self) -> Self {
        self.mount(
            "/items",
            routes![
                get_all_items,
                get_item_by_id,
                create_item,
                update_item,
                delete_item
            ],
        )
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct ItemResponse {
    pub item_type: String,
    pub id: i32,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct CreateItemRequest {
    pub item_type: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct UpdateItemRequest {
    pub item_type: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct ApiResponse {
    pub message: String,
}

/// Get all items
#[utoipa::path(
    get,
    path = "/items",
    tags = ["items"],  // Add this line to assign tag
    responses(
        (status = 200, description = "List all items successfully", body = [ItemResponse])
    )
)]
#[get("/")]
pub async fn get_all_items(database: &State<DatabaseConnection>) -> Json<Vec<ItemResponse>> {
    // Implementation remains the same
    let db = database as &DatabaseConnection;

    let items = Item::find()
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|i| ItemResponse {
            item_type: i.item_type,
            id: i.id,
        })
        .collect::<Vec<ItemResponse>>();

    Json(items)
}

/// Get item by ID
#[utoipa::path(
    get,
    path = "/items/{id}",
    tags = ["items"],  // Add this line
    params(
        ("id" = i32, Path, description = "Item identifier")
    ),
    responses(
        (status = 200, description = "Item found successfully", body = ItemResponse),
        (status = 404, description = "Item not found", body = ApiResponse)
    )
)]
#[get("/<id>")]
pub async fn get_item_by_id(
    id: i32,
    database: &State<DatabaseConnection>,
) -> Result<Json<ItemResponse>, NotFound<Json<ApiResponse>>> {
    // Implementation remains the same
    let db = database as &DatabaseConnection;

    match Item::find_by_id(id).one(db).await {
        Ok(Some(item)) => Ok(Json(ItemResponse {
            item_type: item.item_type,
            id: item.id,
        })),
        _ => Err(NotFound(Json(ApiResponse {
            message: format!("Item with id {} not found", id),
        }))),
    }
}

/// Create a new item
#[utoipa::path(
    post,
    path = "/items",
    tags = ["items"],  // Add this line
    request_body = CreateItemRequest,
    responses(
        (status = 201, description = "Item created successfully", body = ItemResponse)
    )
)]
#[post("/", data = "<item_data>")]
pub async fn create_item(
    item_data: Json<CreateItemRequest>,
    database: &State<DatabaseConnection>,
) -> Created<Json<ItemResponse>> {
    // Implementation remains the same
    let db = database as &DatabaseConnection;

    let new_item = item::ActiveModel {
        item_type: ActiveValue::set(item_data.item_type.clone()),
        ..Default::default()
    };

    let insert_result = new_item.insert(db).await.unwrap();

    Created::new("/").body(Json(ItemResponse {
        item_type: item_data.item_type.clone(),
        id: insert_result.id,
    }))
}

/// Update an existing item
#[utoipa::path(
    put,
    path = "/items/{id}",
    tags = ["items"],  // Add this line
    params(
        ("id" = i32, Path, description = "Item identifier")
    ),
    request_body = UpdateItemRequest,
    responses(
        (status = 200, description = "Item updated successfully", body = ItemResponse),
        (status = 404, description = "Item not found", body = ApiResponse)
    )
)]
#[put("/<id>", data = "<item_data>")]
pub async fn update_item(
    id: i32,
    item_data: Json<UpdateItemRequest>,
    database: &State<DatabaseConnection>,
) -> Result<Json<ItemResponse>, NotFound<Json<ApiResponse>>> {
    // Implementation remains the same
    let db = database as &DatabaseConnection;

    // Find the item to update
    let item_result = Item::find_by_id(id).one(db).await;

    match item_result {
        Ok(Some(item)) => {
            // Create an active model from the found item
            let mut item_active: item::ActiveModel = item.into();

            // Update fields
            item_active.item_type = ActiveValue::set(item_data.item_type.clone());

            // Save changes
            let updated_item = item_active.update(db).await.unwrap();

            Ok(Json(ItemResponse {
                item_type: updated_item.item_type,
                id: updated_item.id,
            }))
        }
        _ => Err(NotFound(Json(ApiResponse {
            message: format!("Item with id {} not found", id),
        }))),
    }
}

/// Delete an item
#[utoipa::path(
    delete,
    path = "/items/{id}",
    tags = ["items"],  // Add this line
    params(
        ("id" = i32, Path, description = "Item identifier")
    ),
    responses(
        (status = 204, description = "Item deleted successfully"),
        (status = 404, description = "Item not found", body = ApiResponse)
    )
)]
#[delete("/<id>")]
pub async fn delete_item(
    id: i32,
    database: &State<DatabaseConnection>,
) -> Result<Status, NotFound<Json<ApiResponse>>> {
    // Implementation remains the same
    let db = database as &DatabaseConnection;

    // Find the item to delete
    let item_result = Item::find_by_id(id).one(db).await;

    match item_result {
        Ok(Some(item)) => {
            // Delete the item
            item.delete(db).await.unwrap();
            Ok(Status::NoContent)
        }
        _ => Err(NotFound(Json(ApiResponse {
            message: format!("Item with id {} not found", id),
        }))),
    }
}

// Create the OpenAPI documentation using the utoipa macro
#[derive(OpenApi)]
#[openapi(
    paths(
        get_all_items,
        get_item_by_id,
        create_item,
        update_item,
        delete_item
    ),
    components(
        schemas(ItemResponse, CreateItemRequest, UpdateItemRequest, ApiResponse)
    ),
    tags(
        (name = "items", description = "Item management API")
    )
)]
pub struct ItemApiDoc;
