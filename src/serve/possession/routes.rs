use crate::db::entities::{possession, prelude::*};
use rocket::{
    Build, Rocket, State, delete, get,
    http::Status,
    post, put,
    response::status::{Created, NotFound},
    routes,
    serde::{Deserialize, Serialize, json::Json},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter,
};

pub trait PossessionRoutes {
    fn mount_possessions(self) -> Self;
}

impl PossessionRoutes for Rocket<Build> {
    fn mount_possessions(self) -> Self {
        self.mount(
            "/possessions",
            routes![
                get_all_possessions,
                get_possession_by_id,
                create_possession,
                update_possession,
                delete_possession,
                get_possessions_by_owner,
                get_possessions_by_item
            ],
        )
    }
}

// Response model with related data
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PossessionResponse {
    pub id: i32,
    pub owner_id: i32,
    pub item_id: i32,
    pub item_type: Option<String>, // Include item data
}

// Request model for creating a possession
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreatePossessionRequest {
    pub owner_id: i32,
    pub item_id: i32,
}

// Request model for updating a possession
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UpdatePossessionRequest {
    pub owner_id: i32,
    pub item_id: i32,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ApiResponse {
    pub message: String,
}

// GET /possessions - Get all possessions
#[get("/")]
pub async fn get_all_possessions(
    database: &State<DatabaseConnection>,
) -> Json<Vec<PossessionResponse>> {
    let db = database as &DatabaseConnection;

    // Find all possessions with their related items
    let possessions = Possession::find()
        .find_with_related(Item)
        .all(db)
        .await
        .unwrap_or_default();

    // Map to response format
    let responses = possessions
        .into_iter()
        .map(|(p, items)| {
            // Get the first (and only) related item
            let item_type = items.first().map(|i| i.item_type.clone());

            PossessionResponse {
                id: p.id,
                owner_id: p.owner,
                item_id: p.item,
                item_type,
            }
        })
        .collect();

    Json(responses)
}

// GET /possessions/<id> - Get possession by ID
#[get("/<id>")]
pub async fn get_possession_by_id(
    id: i32,
    database: &State<DatabaseConnection>,
) -> Result<Json<PossessionResponse>, NotFound<Json<ApiResponse>>> {
    let db = database as &DatabaseConnection;

    // Try to find possession with given ID
    let possession_result = Possession::find_by_id(id)
        .find_with_related(Item)
        .all(db)
        .await;

    match possession_result {
        Ok(results) if !results.is_empty() => {
            let (possession, items) = &results[0];

            // Get item type if item exists
            let item_type = items.first().map(|i| i.item_type.clone());

            Ok(Json(PossessionResponse {
                id: possession.id,
                owner_id: possession.owner,
                item_id: possession.item,
                item_type,
            }))
        }
        _ => Err(NotFound(Json(ApiResponse {
            message: format!("Possession with id {} not found", id),
        }))),
    }
}

// POST /possessions - Create a new possession
#[post("/", data = "<possession_data>")]
pub async fn create_possession(
    possession_data: Json<CreatePossessionRequest>,
    database: &State<DatabaseConnection>,
) -> Result<Created<Json<PossessionResponse>>, Json<ApiResponse>> {
    let db = database as &DatabaseConnection;

    // Validate owner and item exist
    let owner_exists = Owner::find_by_id(possession_data.owner_id)
        .one(db)
        .await
        .unwrap_or(None)
        .is_some();
    let item_exists = Item::find_by_id(possession_data.item_id)
        .one(db)
        .await
        .unwrap_or(None);

    if !owner_exists {
        return Err(Json(ApiResponse {
            message: format!("Owner with id {} not found", possession_data.owner_id),
        }));
    }

    if item_exists.is_none() {
        return Err(Json(ApiResponse {
            message: format!("Item with id {} not found", possession_data.item_id),
        }));
    }

    // Create active model
    let new_possession = possession::ActiveModel {
        owner: ActiveValue::set(possession_data.owner_id),
        item: ActiveValue::set(possession_data.item_id),
        ..Default::default()
    };

    // Insert and get the created possession
    let insert_result = new_possession.insert(db).await.unwrap();

    // Return with 201 Created status
    Ok(
        Created::new(format!("/possessions/{}", insert_result.id)).body(Json(PossessionResponse {
            id: insert_result.id,
            owner_id: insert_result.owner,
            item_id: insert_result.item,
            item_type: item_exists.map(|i| i.item_type),
        })),
    )
}

// PUT /possessions/<id> - Update a possession
#[put("/<id>", data = "<possession_data>")]
pub async fn update_possession(
    id: i32,
    possession_data: Json<UpdatePossessionRequest>,
    database: &State<DatabaseConnection>,
) -> Result<Json<PossessionResponse>, NotFound<Json<ApiResponse>>> {
    let db = database as &DatabaseConnection;

    // Validate owner and item exist
    let owner_exists = Owner::find_by_id(possession_data.owner_id)
        .one(db)
        .await
        .unwrap_or(None)
        .is_some();
    let item_exists = Item::find_by_id(possession_data.item_id)
        .one(db)
        .await
        .unwrap_or(None);

    if !owner_exists {
        return Err(NotFound(Json(ApiResponse {
            message: format!("Owner with id {} not found", possession_data.owner_id),
        })));
    }

    if item_exists.is_none() {
        return Err(NotFound(Json(ApiResponse {
            message: format!("Item with id {} not found", possession_data.item_id),
        })));
    }

    // Find the possession to update
    let possession_result = Possession::find_by_id(id).one(db).await;

    match possession_result {
        Ok(Some(possession)) => {
            // Create an active model from the found possession
            let mut possession_active: possession::ActiveModel = possession.into();

            // Update fields
            possession_active.owner = ActiveValue::set(possession_data.owner_id);
            possession_active.item = ActiveValue::set(possession_data.item_id);

            // Save changes
            let updated_possession = possession_active.update(db).await.unwrap();

            Ok(Json(PossessionResponse {
                id: updated_possession.id,
                owner_id: updated_possession.owner,
                item_id: updated_possession.item,
                item_type: item_exists.map(|i| i.item_type),
            }))
        }
        _ => Err(NotFound(Json(ApiResponse {
            message: format!("Possession with id {} not found", id),
        }))),
    }
}

// DELETE /possessions/<id> - Delete a possession
#[delete("/<id>")]
pub async fn delete_possession(
    id: i32,
    database: &State<DatabaseConnection>,
) -> Result<Status, NotFound<Json<ApiResponse>>> {
    let db = database as &DatabaseConnection;

    // Find the possession to delete
    let possession_result = Possession::find_by_id(id).one(db).await;

    match possession_result {
        Ok(Some(possession)) => {
            // Delete the possession
            possession.delete(db).await.unwrap();
            Ok(Status::NoContent)
        }
        _ => Err(NotFound(Json(ApiResponse {
            message: format!("Possession with id {} not found", id),
        }))),
    }
}

// Additional helper endpoints for relationships

// GET /possessions/owner/<owner_id> - Get all possessions for an owner
#[get("/owner/<owner_id>")]
pub async fn get_possessions_by_owner(
    owner_id: i32,
    database: &State<DatabaseConnection>,
) -> Result<Json<Vec<PossessionResponse>>, NotFound<Json<ApiResponse>>> {
    let db = database as &DatabaseConnection;

    // Check if owner exists
    let owner_exists = Owner::find_by_id(owner_id)
        .one(db)
        .await
        .unwrap_or(None)
        .is_some();

    if !owner_exists {
        return Err(NotFound(Json(ApiResponse {
            message: format!("Owner with id {} not found", owner_id),
        })));
    }

    // Find possessions by owner
    let possessions = Possession::find()
        .filter(possession::Column::Owner.eq(owner_id))
        .find_with_related(Item)
        .all(db)
        .await
        .unwrap_or_default();

    // Map to response format
    let responses = possessions
        .into_iter()
        .map(|(p, items)| {
            let item_type = items.first().map(|i| i.item_type.clone());

            PossessionResponse {
                id: p.id,
                owner_id: p.owner,
                item_id: p.item,
                item_type,
            }
        })
        .collect();

    Ok(Json(responses))
}

// GET /possessions/item/<item_id> - Get all possessions for an item
#[get("/item/<item_id>")]
pub async fn get_possessions_by_item(
    item_id: i32,
    database: &State<DatabaseConnection>,
) -> Result<Json<Vec<PossessionResponse>>, NotFound<Json<ApiResponse>>> {
    let db = database as &DatabaseConnection;

    // Check if item exists
    let item_exists = Item::find_by_id(item_id).one(db).await.unwrap_or(None);

    if item_exists.is_none() {
        return Err(NotFound(Json(ApiResponse {
            message: format!("Item with id {} not found", item_id),
        })));
    }

    // Find possessions by item
    let possessions = Possession::find()
        .filter(possession::Column::Item.eq(item_id))
        .all(db)
        .await
        .unwrap_or_default();

    // Map to response format
    let responses = possessions
        .into_iter()
        .map(|p| PossessionResponse {
            id: p.id,
            owner_id: p.owner,
            item_id: p.item,
            item_type: item_exists.as_ref().map(|i| i.item_type.clone()),
        })
        .collect();

    Ok(Json(responses))
}
