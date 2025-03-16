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

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ItemResponse {
    pub item_type: String,
    pub id: i32,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateItemRequest {
    pub item_type: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UpdateItemRequest {
    pub item_type: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ApiResponse {
    pub message: String,
}

// GET /items - Get all items
#[get("/")]
pub async fn get_all_items(database: &State<DatabaseConnection>) -> Json<Vec<ItemResponse>> {
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

// GET /items/<id> - Get item by ID
#[get("/<id>")]
pub async fn get_item_by_id(
    id: i32,
    database: &State<DatabaseConnection>,
) -> Result<Json<ItemResponse>, NotFound<Json<ApiResponse>>> {
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

// POST /items - Create a new item
#[post("/", data = "<item_data>")]
pub async fn create_item(
    item_data: Json<CreateItemRequest>,
    database: &State<DatabaseConnection>,
) -> Created<Json<ItemResponse>> {
    let db = database as &DatabaseConnection;

    // Create active model
    let new_item = item::ActiveModel {
        item_type: ActiveValue::set(item_data.item_type.clone()),
        ..Default::default()
    };

    // Insert and get the created item
    let insert_result = new_item.insert(db).await.unwrap();

    // Return with 201 Created status
    Created::new("/").body(Json(ItemResponse {
        item_type: item_data.item_type.clone(),
        id: insert_result.id,
    }))
}

// PUT /items/<id> - Update an item
#[put("/<id>", data = "<item_data>")]
pub async fn update_item(
    id: i32,
    item_data: Json<UpdateItemRequest>,
    database: &State<DatabaseConnection>,
) -> Result<Json<ItemResponse>, NotFound<Json<ApiResponse>>> {
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

// DELETE /items/<id> - Delete an item
#[delete("/<id>")]
pub async fn delete_item(
    id: i32,
    database: &State<DatabaseConnection>,
) -> Result<Status, NotFound<Json<ApiResponse>>> {
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
