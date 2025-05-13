use crate::db::{self, DbPool, NewOrderRequest, Order, OrderStatus, OrderType, DbError};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::fmt;
use tokio::sync::RwLock; // For thread-safe access to the cache

// --- Engine Error Definition ---
#[derive(Debug)]
pub enum EngineError {
    Validation(String),
    Database(DbError),
    OrderNotFound(Uuid),
    UpdateConflict(String),
    CacheError(String), // For cache-specific issues, though unlikely with RwLock directly
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EngineError::Validation(msg) => write!(f, "Validation Error: {}", msg),
            EngineError::Database(db_err) => write!(f, "Database Error: {:?}", db_err),
            EngineError::OrderNotFound(order_id) => write!(f, "Order not found: {}", order_id),
            EngineError::UpdateConflict(msg) => write!(f, "Update Conflict: {}", msg),
            EngineError::CacheError(msg) => write!(f, "Cache Error: {}", msg),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<DbError> for EngineError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::NotFound => EngineError::OrderNotFound(Uuid::nil()), 
            _ => EngineError::Database(err),
        }
    }
}

// --- Trading Engine Definition ---
#[derive(Debug)]
pub struct TradingEngine {
    db_pool: DbPool,
    order_cache: RwLock<HashMap<Uuid, Order>>,
}

impl TradingEngine {
    pub fn new(db_pool: DbPool) -> Self {
        TradingEngine {
            db_pool,
            order_cache: RwLock::new(HashMap::new()),
        }
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

    /// Retrieves an order, first checking cache, then DB. Populates cache if fetched from DB.
    pub async fn get_order(&self, order_id: Uuid) -> Result<Option<Order>, EngineError> {
        // 1. Try to get from cache with a read lock
        let cached_order = self.order_cache.read().await.get(&order_id).cloned();
        if cached_order.is_some() {
            // println!("[Cache HIT] Order {} found in cache.", order_id);
            return Ok(cached_order);
        }

        // println!("[Cache MISS] Order {} not in cache, fetching from DB.", order_id);
        // 2. If not in cache, fetch from DB
        match db::get_order_by_id(&self.db_pool, order_id).await {
            Ok(Some(db_order)) => {
                // 3. Populate cache with a write lock
                let mut cache_writer = self.order_cache.write().await;
                cache_writer.insert(order_id, db_order.clone());
                // println!("[Cache POPULATE] Order {} added to cache.", order_id);
                Ok(Some(db_order))
            }
            Ok(None) => Ok(None), // Order not found in DB either
            Err(db_err) => Err(EngineError::from(db_err)),
        }
    }

    pub async fn submit_new_order(&self, order_request: &NewOrderRequest) -> Result<Order, EngineError> {
        Self::validate_new_order_request(order_request)?;
        
        let order_id = db::create_order(&self.db_pool, order_request, OrderStatus::PendingValidation).await?;
        
        // Fetch the order from DB to ensure we have the full, persisted object
        match db::get_order_by_id(&self.db_pool, order_id).await? {
            Some(persisted_order) => {
                // Add to cache
                let mut cache_writer = self.order_cache.write().await;
                cache_writer.insert(order_id, persisted_order.clone());
                // println!("[Cache POPULATE on submit] Order {} added to cache.", order_id);
                Ok(persisted_order)
            }
            None => Err(EngineError::OrderNotFound(order_id)), // Should not happen if create succeeded
        }
    }

    pub async fn confirm_order(&self, order_id: Uuid, new_status: OrderStatus) -> Result<Order, EngineError> {
        // 1. Get the current order (will use cache if available, or fetch from DB and populate cache)
        let order = match self.get_order(order_id).await? {
            Some(o) => o,
            None => return Err(EngineError::OrderNotFound(order_id)),
        };

        if order.order_status != OrderStatus::PendingValidation && new_status == OrderStatus::New {
            return Err(EngineError::UpdateConflict(
                format!("Order {} cannot be moved to New from status {:?}. Expected PendingValidation.", order_id, order.order_status)
            ));
        }

        match db::update_order_status(&self.db_pool, order_id, new_status).await {
            Ok(0) => Err(EngineError::OrderNotFound(order_id)),
            Ok(_) => {
                // Fetch the updated order to get its new state (e.g., updated_at)
                match db::get_order_by_id(&self.db_pool, order_id).await? {
                    Some(updated_db_order) => {
                        // Update the cache with the new version of the order
                        let mut cache_writer = self.order_cache.write().await;
                        cache_writer.insert(order_id, updated_db_order.clone());
                        // println!("[Cache UPDATE on confirm] Order {} updated in cache.", order_id);
                        Ok(updated_db_order)
                    }
                    None => Err(EngineError::OrderNotFound(order_id)), 
                }
            }
            Err(e) => Err(EngineError::from(e)),
        }
    }
}
