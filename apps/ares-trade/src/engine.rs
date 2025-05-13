use crate::db::{self, DbPool, NewOrderRequest, Order, OrderStatus, OrderType, DbError};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::fmt;

// --- Engine Error Definition ---
#[derive(Debug)]
pub enum EngineError {
    Validation(String),
    Database(DbError),
    OrderNotFound(Uuid),
    UpdateConflict(String), // Added for cases like status transition errors
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EngineError::Validation(msg) => write!(f, "Validation Error: {}", msg),
            EngineError::Database(db_err) => write!(f, "Database Error: {:?}", db_err),
            EngineError::OrderNotFound(order_id) => write!(f, "Order not found: {}", order_id),
            EngineError::UpdateConflict(msg) => write!(f, "Update Conflict: {}", msg),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<DbError> for EngineError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::NotFound => EngineError::OrderNotFound(Uuid::nil()), // Generic, might need context for specific ID
            _ => EngineError::Database(err),
        }
    }
}

// --- Trading Engine Definition ---
#[derive(Debug)]
pub struct TradingEngine {
    db_pool: DbPool,
}

impl TradingEngine {
    pub fn new(db_pool: DbPool) -> Self {
        TradingEngine { db_pool }
    }

    fn validate_new_order_request(order_request: &NewOrderRequest) -> Result<(), EngineError> {
        if order_request.order_quantity <= Decimal::ZERO {
            return Err(EngineError::Validation("Order quantity must be positive.".to_string()));
        }
        if order_request.order_type == OrderType::Limit {
            if order_request.price.is_none() || order_request.price.map_or(false, |p| p <= Decimal::ZERO) {
                return Err(EngineError::Validation("Price must be set and positive for a Limit order.".to_string()));
            }
        }
        Ok(())
    }

    pub async fn submit_new_order(&self, order_request: &NewOrderRequest) -> Result<Order, EngineError> {
        Self::validate_new_order_request(order_request)?;
        let order_id = db::create_order(&self.db_pool, order_request, OrderStatus::PendingValidation).await?;
        match db::get_order_by_id(&self.db_pool, order_id).await? {
            Some(order) => Ok(order),
            None => Err(EngineError::OrderNotFound(order_id)), 
        }
    }

    pub async fn confirm_order(&self, order_id: Uuid, new_status: OrderStatus) -> Result<Order, EngineError> {
        // 1. Get the current order to check its status before updating
        let order = match db::get_order_by_id(&self.db_pool, order_id).await? {
            Some(o) => o,
            None => return Err(EngineError::OrderNotFound(order_id)),
        };

        // 2. Add any state transition validation if needed (e.g., only PendingValidation can go to New)
        if order.order_status != OrderStatus::PendingValidation && new_status == OrderStatus::New {
            return Err(EngineError::UpdateConflict(
                format!("Order {} cannot be moved to New from status {:?}. Expected PendingValidation.", order_id, order.order_status)
            ));
        }
        // Add more specific transition rules as the engine evolves

        // 3. Update the order status in the database
        // Consider adding last_updated_by here if that field is added to db::update_order_status
        match db::update_order_status(&self.db_pool, order_id, new_status).await {
            Ok(0) => {
                // This case should ideally be caught by the initial get_order_by_id, 
                // but as a safeguard if something changed between get and update.
                Err(EngineError::OrderNotFound(order_id))
            }
            Ok(_) => { // Rows affected > 0
                // 4. Fetch the updated order to return its new state
                match db::get_order_by_id(&self.db_pool, order_id).await? {
                    Some(updated_order) => Ok(updated_order),
                    None => Err(EngineError::OrderNotFound(order_id)), // Should not happen if update succeeded
                }
            }
            Err(e) => Err(EngineError::from(e)), // Convert DbError to EngineError
        }
    }
}
