use crate::db::entities::{possession, prelude::*};
use crate::serve::trade::logic::{Trade, TradeLogic, TradesMutex};
use rocket::{
    Build, Rocket, State,
    delete, get, post, put,
    http::Status,
    response::status::{Created, NotFound},
    routes,
    serde::{Deserialize, Serialize, json::Json},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, DbErr, TransactionTrait
};
use std::sync::atomic::{AtomicU64, Ordering};
use utoipa::{ToSchema, OpenApi};

pub trait TradeRoutes {
    fn mount_trades(self) -> Self;
}

impl TradeRoutes for Rocket<Build> {
    fn mount_trades(self) -> Self {
        self.mount(
            "/trades",
            routes![
                get_all_trades,
                get_trade_by_id,
                create_trade,
                add_item_to_trade,
                remove_item_from_trade,
                accept_trade,
                cancel_trade,
            ],
        )
    }
}

// Unique ID generator for trades
static NEXT_TRADE_ID: AtomicU64 = AtomicU64::new(1);

// Response model for trades
#[derive(Serialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct TradeResponse {
    pub id: u64,
    pub trader_1_id: i32,
    pub trader_1_items: Vec<i32>,
    pub trader_1_accept: bool,
    pub trader_2_id: i32,
    pub trader_2_items: Vec<i32>,
    pub trader_2_accept: bool,
}

// Request model for creating a trade
#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct CreateTradeRequest {
    pub trader_1_id: i32,
    pub trader_2_id: i32,
}

// Request model for adding/removing items
#[derive(Deserialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct TradeItemRequest {
    pub owner_id: i32,
    pub item_id: i32,
}

#[derive(Serialize, ToSchema)]
#[serde(crate = "rocket::serde")]
pub struct ApiResponse {
    pub message: String,
}

// Helper function to execute a trade
async fn execute_trade_internal(
    trade: &Trade,
    db: &DatabaseConnection
) -> Result<(), DbErr> {
    // Start a transaction
    let txn = db.begin().await?;
    
    // Update ownership of all items from trader 1 to trader 2
    for item_id in &trade.trade_1_items {
        let possession_result = Possession::find_by_id(*item_id).one(db).await?;
        
        if let Some(possession) = possession_result {
            let mut active_model: possession::ActiveModel = possession.into();
            active_model.owner = ActiveValue::set(trade.trader_2.id);
            active_model.update(db).await?;
        }
    }
    
    // Update ownership of all items from trader 2 to trader 1
    for item_id in &trade.trade_2_items {
        let possession_result = Possession::find_by_id(*item_id).one(db).await?;
        
        if let Some(possession) = possession_result {
            let mut active_model: possession::ActiveModel = possession.into();
            active_model.owner = ActiveValue::set(trade.trader_1.id);
            active_model.update(db).await?;
        }
    }
    
    // Commit the transaction
    txn.commit().await?;
    
    Ok(())
}

// GET /trades - Get all trades
#[utoipa::path(
    get,
    path = "/trades",
    tags = ["trades"],
    responses(
        (status = 200, description = "List all trades successfully", body = [TradeResponse])
    )
)]
#[get("/")]
pub async fn get_all_trades(
    trades: &State<TradesMutex>,
) -> Json<Vec<TradeResponse>> {
    let trades_lock = trades.lock().await;
    
    let responses = trades_lock.iter().map(|(_ ,trade)| {
        TradeResponse {
            id: trade.id,
            trader_1_id: trade.trader_1.id,
            trader_1_items: trade.trade_1_items.clone(),
            trader_1_accept: trade.trade_1_accept,
            trader_2_id: trade.trader_2.id,
            trader_2_items: trade.trade_2_items.clone(),
            trader_2_accept: trade.trade_2_accept,
        }
    }).collect();
    
    Json(responses)
}

// GET /trades/<id> - Get trade by ID
#[utoipa::path(
    get,
    path = "/trades/{id}",
    tags = ["trades"],
    params(
        ("id" = u64, Path, description = "Trade identifier")
    ),
    responses(
        (status = 200, description = "Trade found successfully", body = TradeResponse),
        (status = 404, description = "Trade not found", body = ApiResponse)
    )
)]
#[get("/<id>")]
pub async fn get_trade_by_id(
    id: u64,
    trades: &State<TradesMutex>,
) -> Result<Json<TradeResponse>, NotFound<Json<ApiResponse>>> {

    let mut trades_lock = trades.lock().await;
    let trade_opt= trades_lock.get_mut(&id);
    
    match trade_opt {
        Some(trade) => {
            Ok(Json(TradeResponse {
                id: trade.id,
                trader_1_id: trade.trader_1.id,
                trader_1_items: trade.trade_1_items.clone(),
                trader_1_accept: trade.trade_1_accept,
                trader_2_id: trade.trader_2.id,
                trader_2_items: trade.trade_2_items.clone(),
                trader_2_accept: trade.trade_2_accept,
            }))
        },
        None => Err(NotFound(Json(ApiResponse {
            message: format!("Trade with id {} not found", id),
        }))),
    }
}

// POST /trades - Create a new trade
#[utoipa::path(
    post,
    path = "/trades",
    tags = ["trades"],
    request_body = CreateTradeRequest,
    responses(
        (status = 201, description = "Trade created successfully", body = TradeResponse),
        (status = 400, description = "Invalid request data", body = ApiResponse)
    )
)]
#[post("/", data = "<trade_data>")]
pub async fn create_trade(
    trade_data: Json<CreateTradeRequest>,
    trades: &State<TradesMutex>,
    database: &State<DatabaseConnection>,
) -> Result<Created<Json<TradeResponse>>, Json<ApiResponse>> {
    let db = database as &DatabaseConnection;
    
    // Validate both traders exist
    let trader_1 = match Owner::find_by_id(trade_data.trader_1_id).one(db).await {
        Ok(Some(owner)) => owner,
        _ => return Err(Json(ApiResponse {
            message: format!("Trader with id {} not found", trade_data.trader_1_id),
        })),
    };
    
    let trader_2 = match Owner::find_by_id(trade_data.trader_2_id).one(db).await {
        Ok(Some(owner)) => owner,
        _ => return Err(Json(ApiResponse {
            message: format!("Trader with id {} not found", trade_data.trader_2_id),
        })),
    };
    
    // Make sure traders are different
    if trader_1.id == trader_2.id {
        return Err(Json(ApiResponse {
            message: "Cannot create trade with same trader on both sides".to_string(),
        }));
    }
    
    // Get new ID
    let trade_id = NEXT_TRADE_ID.fetch_add(1, Ordering::SeqCst);
    
    // Create new trade
    let new_trade = Trade {
        id: trade_id,
        trader_1,
        trade_1_accept: false,
        trade_1_items: Vec::new(),
        trader_2,
        trade_2_accept: false,
        trade_2_items: Vec::new(),
    };
    
    // Add to collection
    
    let trade_res = Created::new(format!("/trades/{}", trade_id)).body(Json(TradeResponse {
        id: new_trade.id,
        trader_1_id: new_trade.trader_1.id,
        trader_1_items: new_trade.trade_1_items.clone(),
        trader_1_accept: new_trade.trade_1_accept,
        trader_2_id: new_trade.trader_2.id,
        trader_2_items: new_trade.trade_2_items.clone(),
        trader_2_accept: new_trade.trade_2_accept,
    }));
    
    let mut trades_lock = trades.lock().await;
    trades_lock.insert(new_trade.id, new_trade);
    drop(trades_lock);
    
    Ok(trade_res)
}

// POST /trades/<id>/add-item - Add item to trade
#[utoipa::path(
    post,
    path = "/trades/{id}/add-item",
    tags = ["trades"],
    params(
        ("id" = u64, Path, description = "Trade identifier")
    ),
    request_body = TradeItemRequest,
    responses(
        (status = 200, description = "Item added to trade successfully", body = TradeResponse),
        (status = 400, description = "Invalid request data", body = ApiResponse),
        (status = 404, description = "Trade, owner or possession not found", body = ApiResponse)
    )
)]
#[post("/<id>/add-item", data = "<item_data>")]
pub async fn add_item_to_trade(
    id: u64,
    item_data: Json<TradeItemRequest>,
    trades: &State<TradesMutex>,
    database: &State<DatabaseConnection>,
) -> Result<Json<TradeResponse>, Status> {
    let db = database as &DatabaseConnection;
    
    // Find owner and possession
    let owner = match Owner::find_by_id(item_data.owner_id).one(db).await {
        Ok(Some(owner)) => owner,
        _ => return Err(Status::NotFound),
    };
    
    let possession = match Possession::find_by_id(item_data.item_id).one(db).await {
        Ok(Some(possession)) => possession,
        _ => return Err(Status::NotFound),
    };
    
    // Verify ownership
    if possession.owner != owner.id {
        return Err(Status::BadRequest);
    }
    
    // Find and update trade
    let mut trades_lock = trades.lock().await;
    let trade_opt= trades_lock.get_mut(&id);
    
    match trade_opt {
        Some(trade) => {
            
            // Verify trader is part of trade
            if trade.trader_1.id != owner.id && trade.trader_2.id != owner.id {
                return Err(Status::BadRequest);
            }
            
            // Add item to trade
            if trade.add_to_trade(&owner, &possession) {
                Ok(Json(TradeResponse {
                    id: trade.id,
                    trader_1_id: trade.trader_1.id,
                    trader_1_items: trade.trade_1_items.clone(),
                    trader_1_accept: trade.trade_1_accept,
                    trader_2_id: trade.trader_2.id,
                    trader_2_items: trade.trade_2_items.clone(),
                    trader_2_accept: trade.trade_2_accept,
                }))
            } else {
                Err(Status::InternalServerError)
            }
        },
        None => Err(Status::NotFound),
    }
}

// DELETE /trades/<id>/remove-item - Remove item from trade
#[utoipa::path(
    delete,
    path = "/trades/{id}/remove-item",
    tags = ["trades"],
    params(
        ("id" = u64, Path, description = "Trade identifier")
    ),
    request_body = TradeItemRequest,
    responses(
        (status = 200, description = "Item removed from trade successfully", body = TradeResponse),
        (status = 400, description = "Invalid request data", body = ApiResponse),
        (status = 404, description = "Trade, owner or possession not found", body = ApiResponse)
    )
)]
#[delete("/<id>/remove-item", data = "<item_data>")]
pub async fn remove_item_from_trade(
    id: u64,
    item_data: Json<TradeItemRequest>,
    trades: &State<TradesMutex>,
    database: &State<DatabaseConnection>,
) -> Result<Json<TradeResponse>, Status> {
    let db = database as &DatabaseConnection;
    
    // Find owner and possession
    let owner = match Owner::find_by_id(item_data.owner_id).one(db).await {
        Ok(Some(owner)) => owner,
        _ => return Err(Status::NotFound),
    };
    
    let possession = match Possession::find_by_id(item_data.item_id).one(db).await {
        Ok(Some(possession)) => possession,
        _ => return Err(Status::NotFound),
    };
    
    // Find and update trade
    let mut trades_lock = trades.lock().await;
    let trade_opt= trades_lock.get_mut(&id);
    
    match trade_opt {
        Some(trade) => {
            
            // Verify trader is part of trade
            if trade.trader_1.id != owner.id && trade.trader_2.id != owner.id {
                return Err(Status::BadRequest);
            }
            
            // Remove item from trade
            if trade.remove_from_trade(&owner, &possession) {
                Ok(Json(TradeResponse {
                    id: trade.id,
                    trader_1_id: trade.trader_1.id,
                    trader_1_items: trade.trade_1_items.clone(),
                    trader_1_accept: trade.trade_1_accept,
                    trader_2_id: trade.trader_2.id,
                    trader_2_items: trade.trade_2_items.clone(),
                    trader_2_accept: trade.trade_2_accept,
                }))
            } else {
                Err(Status::BadRequest)
            }
        },
        None => Err(Status::NotFound),
    }
}

// PUT /trades/<id>/accept - Accept trade
#[utoipa::path(
    put,
    path = "/trades/{id}/accept",
    tags = ["trades"],
    params(
        ("id" = u64, Path, description = "Trade identifier"),
        ("owner_id" = i32, Query, description = "Owner accepting the trade")
    ),
    responses(
        (status = 200, description = "Trade status updated successfully", body = TradeResponse),
        (status = 201, description = "Trade executed successfully", body = ApiResponse),
        (status = 404, description = "Trade or owner not found", body = ApiResponse),
        (status = 500, description = "Error executing trade", body = ApiResponse)
    )
)]
#[put("/<id>/accept?<owner_id>")]
pub async fn accept_trade(
    id: u64,
    owner_id: i32,
    trades: &State<TradesMutex>,
    database: &State<DatabaseConnection>,
) -> Result<Json<ApiResponse>, Status> {
    let db = database as &DatabaseConnection;
    
    // Find owner
    let owner = match Owner::find_by_id(owner_id).one(db).await {
        Ok(Some(owner)) => owner,
        _ => return Err(Status::NotFound),
    };
    
    // Find and update trade
    let mut trades_lock = trades.lock().await;
    let trade_opt= trades_lock.get_mut(&id);
    
    match trade_opt {
        Some(trade) => {
            
            // Verify trader is part of trade
            if trade.trader_1.id != owner.id && trade.trader_2.id != owner.id {
                return Err(Status::BadRequest);
            }
            
            // Change trade status
            trade.change_trade_status(&owner);
            
            // Check if both traders have accepted
            if trade.trade_1_accept && trade.trade_2_accept {
                // Both traders have accepted, execute the trade
                if let Err(_) = execute_trade_internal(trade, db).await {
                    return Err(Status::InternalServerError);
                }
                
                // Trade executed successfully, remove it from active trades
                
                // Get a new lock and remove the trade
                if let Some(deleted) = trades_lock.remove(&id){
                    return Ok(Json(ApiResponse {
                        message: format!(
                            "Trade between {} and {} executed successfully", 
                            deleted.trader_1.id, 
                            deleted.trader_2.id
                        ),
                    }));

                }
                else {
                    return Ok(Json(ApiResponse {
                            message: format!(
                            "Trade executed successfully"
                        ),
                    }));
                }
                
            }
            
            // Return the updated trade status
            Ok(Json(ApiResponse {
                message: format!(
                    "Trade acceptance status updated. Trader 1: {}, Trader 2: {}", 
                    if trade.trade_1_accept { "Accepted" } else { "Not accepted" },
                    if trade.trade_2_accept { "Accepted" } else { "Not accepted" }
                ),
            }))
        },
        None => Err(Status::NotFound),
    }
}

// DELETE /trades/<id> - Cancel a trade
#[utoipa::path(
    delete,
    path = "/trades/{id}",
    tags = ["trades"],
    params(
        ("id" = u64, Path, description = "Trade identifier")
    ),
    responses(
        (status = 204, description = "Trade cancelled successfully"),
        (status = 404, description = "Trade not found", body = ApiResponse)
    )
)]
#[delete("/<id>")]
pub async fn cancel_trade(
    id: u64,
    trades: &State<TradesMutex>,
) -> Result<Status, NotFound<Json<ApiResponse>>> {
    let mut trades_lock = trades.lock().await;
    
    if trades_lock.remove(&id).is_some(){
        
        Ok(Status::NoContent)
    }
    else {
        Err(NotFound(Json(ApiResponse {
            message: format!("Trade with id {} not found", id),
        })))
    }
}

// Create the OpenAPI documentation struct
#[derive(OpenApi)]
#[openapi(
    paths(
        get_all_trades,
        get_trade_by_id,
        create_trade,
        add_item_to_trade,
        remove_item_from_trade,
        accept_trade,
        cancel_trade,
    ),
    components(
        schemas(TradeResponse, CreateTradeRequest, TradeItemRequest, ApiResponse)
    ),
    tags(
        (name = "trades", description = "Trade management API")
    )
)]
pub struct TradeApiDoc;