use std::env;
use dotenv::dotenv;
use std::fmt;

// Declare the db module, which will look for db.rs or db/mod.rs
pub mod db;

// Custom error to wrap String errors for main
#[derive(Debug)]
struct MainError(String);

impl fmt::Display for MainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MainError {}

impl From<String> for MainError {
    fn from(err: String) -> MainError {
        MainError(err)
    }
}

// Also allow DbError to be converted if needed, though create_order errors are currently just printed
impl From<db::DbError> for MainError {
    fn from(err: db::DbError) -> MainError {
        MainError(format!("{:?}", err)) // Simple Display for now
    }
}


#[tokio::main]
async fn main() -> Result<(), MainError> { // Changed error type
    // Load environment variables from .env file, if it exists
    dotenv().ok();

    println!("Starting Ares Trading Engine...");

    // Retrieve Database URL from environment variable
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file or environment for Ares to connect to the database.");

    // Create the database connection pool
    let pool = db::create_db_pool(&database_url)?; // Use ? operator

    println!("Successfully connected to the database.");
    println!("Ares Trading Engine initialized.");

    // --- Placeholder for Core OMS Logic ---
    // (Rest of the main function remains similar, error handling in test block might need adjustment)
    if env::var("RUN_DB_TEST").is_ok() {
        println!("Running a simple database test...");
        let test_account_id = uuid::Uuid::new_v4();
        let test_instrument_id = uuid::Uuid::new_v4();

        let new_order_request = db::NewOrderRequest {
            client_order_id: format!("TEST_ORD_{}", chrono::Utc::now().timestamp_millis()),
            account_id: test_account_id,
            instrument_id: test_instrument_id,
            side: db::OrderSide::Buy,
            order_type: db::OrderType::Limit,
            order_quantity: rust_decimal::Decimal::new(100, 0),
            price: Some(rust_decimal::Decimal::new(15050, 2)),
            currency_code: "USD".to_string(),
            time_in_force: Some(db::TimeInForce::Day),
            execution_destination: Some("SIMULATOR".to_string()),
            text: Some("Ares initial test order".to_string()),
        };

        match db::create_order(&pool, &new_order_request, db::OrderStatus::New).await {
            Ok(order_id) => {
                println!("Successfully created test order with ID: {}", order_id);
                match db::get_order_by_id(&pool, order_id).await {
                    Ok(Some(order)) => println!("Successfully retrieved test order: {:?}", order),
                    Ok(None) => eprintln!("Test order {} not found after creation.", order_id),
                    Err(e) => eprintln!("Failed to retrieve test order {}: {:?}", order_id, e),
                }
            }
            Err(e) => eprintln!("Failed to create test order: {:?}", e), // These eprintln calls are fine for now
        }
    }

    println!("Ares Trading Engine shutting down.");
    Ok(())
}
