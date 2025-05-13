use std::env;
use dotenv::dotenv;
use std::fmt;

// Declare modules
pub mod db;
pub mod engine;

use engine::TradingEngine;
use db::{OrderStatus, DbError, NewOrderRequest, OrderSide, OrderType, TimeInForce}; // Consolidate db imports
use engine::EngineError;

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

        let test_account_id = uuid::Uuid::new_v4();
        let test_instrument_id = uuid::Uuid::new_v4();

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
                println!("Successfully SUBMITTED test order via engine. Initial status: {:?}, ID: {}", order.order_status, order.order_id);
                assert_eq!(order.order_status, OrderStatus::PendingValidation);

                println!("
Attempting to CONFIRM order {} to status NEW...", order.order_id);
                match trading_engine.confirm_order(order.order_id, OrderStatus::New).await {
                    Ok(confirmed_order) => {
                        println!("Successfully CONFIRMED order. New status: {:?}, ID: {}", confirmed_order.order_status, confirmed_order.order_id);
                        assert_eq!(confirmed_order.order_status, OrderStatus::New);
                    }
                    Err(e) => eprintln!("Failed to CONFIRM order {}: {}", order.order_id, e),
                }

                // Test attempting to confirm again (which should fail based on current simple logic)
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

        let invalid_order_req = NewOrderRequest {
            client_order_id: format!("INVALID_ORD_{}", chrono::Utc::now().timestamp_millis()),
            account_id: test_account_id,
            instrument_id: test_instrument_id,
            side: OrderSide::Sell,
            order_type: OrderType::Market,
            order_quantity: rust_decimal::Decimal::ZERO, // Invalid quantity
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
