use std::env;
use dotenv::dotenv;
use std::fmt;

// Declare modules
pub mod db;
pub mod engine;

use engine::TradingEngine;
use db::{OrderStatus, DbError, NewOrderRequest, OrderSide, OrderType, TimeInForce, Order};
use engine::EngineError;
use uuid::Uuid;

// Custom error for main to wrap various error types
#[derive(Debug)]
struct MainError(String);

impl fmt::Display for MainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MainError {}

impl<E: std::error::Error + fmt::Display> From<E> for MainError
where
    E: Sized,
{
    fn from(err: E) -> MainError {
        MainError(err.to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
    dotenv().ok();
    println!("Starting Ares Trading Engine...");

    let database_url = env::var("DATABASE_URL")
        .map_err(|e| MainError(format!("DATABASE_URL must be set: {}", e)))?;

    let db_pool = db::create_db_pool(&database_url)
        .map_err(|s| MainError(s))?;
    println!("Successfully connected to the database.");

    let trading_engine = TradingEngine::new(db_pool.clone());
    println!("Ares Trading Engine initialized.");

    if env::var("RUN_ENGINE_TEST").is_ok() {
        println!("
--- Running Trading Engine Test ---");

        let test_account_id = Uuid::new_v4();
        let test_instrument_id = Uuid::new_v4();
        let mut submitted_order_id: Option<Uuid> = None; // To store the ID for later get_order test

        let new_order_req = NewOrderRequest {
            client_order_id: format!("ENGINE_ORD_{}", chrono::Utc::now().timestamp_millis()),
            account_id: test_account_id,
            instrument_id: test_instrument_id,
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            order_quantity: rust_decimal::Decimal::new(100, 0),
            price: Some(rust_decimal::Decimal::new(15050, 2)),
            currency_code: "USD".to_string(),
            time_in_force: Some(TimeInForce::Day),
            execution_destination: Some("SIMULATOR".to_string()),
            text: Some("Ares engine test order".to_string()),
        };

        println!("
Submitting a valid order...");
        match trading_engine.submit_new_order(&new_order_req).await {
            Ok(order) => {
                println!("Successfully SUBMITTED test order. Initial status: {:?}, ID: {}", order.order_status, order.order_id);
                assert_eq!(order.order_status, OrderStatus::PendingValidation);
                submitted_order_id = Some(order.order_id); // Save ID for later

                println!("
Attempting to GET order {} (should be a cache hit if submit cached it).", order.order_id);
                match trading_engine.get_order(order.order_id).await {
                    Ok(Some(cached_order)) => {
                        println!("Successfully GOT order {} from engine. Status: {:?}", cached_order.order_id, cached_order.order_status);
                        assert_eq!(cached_order.order_id, order.order_id);
                        assert_eq!(cached_order.order_status, OrderStatus::PendingValidation);
                    }
                    Ok(None) => eprintln!("ERROR: Submitted order {} not found by get_order.", order.order_id),
                    Err(e) => eprintln!("ERROR: Failed to get_order for {}: {}", order.order_id, e),
                }

                println!("
Attempting to CONFIRM order {} to status NEW...", order.order_id);
                match trading_engine.confirm_order(order.order_id, OrderStatus::New).await {
                    Ok(confirmed_order) => {
                        println!("Successfully CONFIRMED order. New status: {:?}, ID: {}", confirmed_order.order_status, confirmed_order.order_id);
                        assert_eq!(confirmed_order.order_status, OrderStatus::New);

                        println!("
Attempting to GET confirmed order {} (should reflect updated status from cache).", confirmed_order.order_id);
                        match trading_engine.get_order(confirmed_order.order_id).await {
                            Ok(Some(cached_confirmed_order)) => {
                                println!("Successfully GOT confirmed order {} from engine. Status: {:?}", cached_confirmed_order.order_id, cached_confirmed_order.order_status);
                                assert_eq!(cached_confirmed_order.order_status, OrderStatus::New);
                            }
                            Ok(None) => eprintln!("ERROR: Confirmed order {} not found by get_order.", confirmed_order.order_id),
                            Err(e) => eprintln!("ERROR: Failed to get_order for confirmed order {}: {}", confirmed_order.order_id, e),
                        }
                    }
                    Err(e) => eprintln!("Failed to CONFIRM order {}: {}", order.order_id, e),
                }

                println!("
Attempting to re-CONFIRM order {} to status NEW (should fail due to status conflict)...", order.order_id);
                match trading_engine.confirm_order(order.order_id, OrderStatus::New).await {
                    Ok(reconfirmed_order) => {
                        eprintln!("ERROR: Order re-confirmation to NEW succeeded unexpectedly: {:?}", reconfirmed_order);
                    }
                    Err(e) => {
                        println!("Correctly failed to re-CONFIRM order to NEW: {}", e);
                        if !matches!(e, EngineError::UpdateConflict(_)) {
                            eprintln!("ERROR: Re-confirmation failed but not due to UpdateConflict as expected.");
                        }
                    }
                }
            }
            Err(e) => eprintln!("Failed to SUBMIT test order via engine: {}", e),
        }

        // Test get_order for a non-existent order ID (cache miss then DB miss)
        let non_existent_uuid = Uuid::new_v4();
        println!("
Attempting to GET a non-existent order {} (should be cache miss then DB miss).", non_existent_uuid);
        match trading_engine.get_order(non_existent_uuid).await {
            Ok(None) => println!("Correctly found no order for non-existent ID {}.", non_existent_uuid),
            Ok(Some(_)) => eprintln!("ERROR: get_order found an order for non-existent ID {}.", non_existent_uuid),
            Err(e) => eprintln!("ERROR: get_order failed for non-existent ID {}: {}", non_existent_uuid, e),
        }
        
        // If an order was submitted, try to get it again - this should be a cache hit from the initial submit/confirm.
        if let Some(id) = submitted_order_id {
            println!("
Attempting to GET previously submitted/confirmed order {} again (should be a cache hit).", id);
            match trading_engine.get_order(id).await {
                Ok(Some(final_get_order)) => {
                    println!("Successfully GOT order {} again from engine. Status: {:?}", final_get_order.order_id, final_get_order.order_status);
                    // Status could be New (if confirm worked) or still PendingValidation if confirm failed but submit worked.
                }
                Ok(None) => eprintln!("ERROR: Previously submitted order {} not found by final get_order.", id),
                Err(e) => eprintln!("ERROR: Failed to final get_order for {}: {}", id, e),
            }
        }

        let invalid_order_req = NewOrderRequest {
            client_order_id: format!("INVALID_ORD_{}", chrono::Utc::now().timestamp_millis()),
            account_id: test_account_id,
            instrument_id: test_instrument_id,
            side: OrderSide::Sell,
            order_type: OrderType::Market,
            order_quantity: rust_decimal::Decimal::ZERO, 
            price: None,
            currency_code: "USD".to_string(),
            time_in_force: Some(TimeInForce::Day),
            execution_destination: Some("SIMULATOR".to_string()),
            text: Some("Ares engine invalid test order".to_string()),
        };

        println!("
Attempting to submit an INVALID order (zero quantity)...");
        match trading_engine.submit_new_order(&invalid_order_req).await {
            Ok(order) => eprintln!("ERROR: Invalid order was accepted by engine: {:?}", order),
            Err(e) => {
                println!("Correctly failed to submit invalid order: {}", e);
                if !matches!(e, EngineError::Validation(_)) {
                     eprintln!("ERROR: Invalid order submission failed but not due to Validation error as expected.");
                }
            }
        }
        println!("
--- Trading Engine Test Complete ---");
    }

    println!("
Ares Trading Engine operations complete. Shutting down.");
    Ok(())
}
