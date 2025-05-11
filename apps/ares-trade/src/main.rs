use std::env;
use dotenv::dotenv;

// Declare the db module, which will look for db.rs or db/mod.rs
pub mod db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file, if it exists
    dotenv().ok();

    println!("Starting Ares Trading Engine...");

    // Retrieve Database URL from environment variable
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file or environment for Ares to connect to the database.");

    // Create the database connection pool
    let pool = match db::create_db_pool(&database_url) {
        Ok(p) => {
            println!("Successfully connected to the database.");
            p
        }
        Err(e) => {
            eprintln!("Failed to create database pool: {}", e);
            // Depending on desired behavior, might panic or exit more gracefully
            return Err(Box::new(e));
        }
    };

    println!("Ares Trading Engine initialized.");

    // --- Placeholder for Core OMS Logic ---
    // In a real application, you would start your API server here (e.g., using Axum, Actix-web, or Tonic),
    // or initialize a message queue consumer (e.g., Kafka, RabbitMQ) to receive order requests.

    // Example: Test creating and retrieving an order (remove or adapt for production)
    // This is just for demonstration. Real order creation would come from an external request.
    if env::var("RUN_DB_TEST").is_ok() {
        println!("Running a simple database test...");
        let test_account_id = uuid::Uuid::new_v4(); // In a real scenario, this ID would exist in the 'accounts' table
        let test_instrument_id = uuid::Uuid::new_v4(); // In a real scenario, this ID would exist in the 'instruments' table

        let new_order_request = db::NewOrderRequest {
            client_order_id: format!("TEST_ORD_{}", chrono::Utc::now().timestamp_millis()),
            account_id: test_account_id,
            instrument_id: test_instrument_id,
            side: db::OrderSide::Buy,
            order_type: db::OrderType::Limit,
            order_quantity: rust_decimal::Decimal::new(100, 0), // 100 units
            price: Some(rust_decimal::Decimal::new(15050, 2)), // Price 150.50
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
            Err(e) => eprintln!("Failed to create test order: {:?}", e),
        }
    }

    // Keep the service running (e.g., if it's an API server)
    // For a simple test like this, it will exit after the test.
    // If this were a server, you'd have something like:
    // api_server::start(pool).await?;

    println!("Ares Trading Engine shutting down.");
    Ok(())
}
