use postgres::{Client, NoTls, Error as PostgresError, Row as PostgresRow, config::Config as PgConfig};
use std::str::FromStr; // Add this for PgConfig::from_str
use r2d2;
use r2d2_postgres::PostgresConnectionManager;
use uuid::Uuid;
use rust_decimal::Decimal; // For NUMERIC types
use chrono::{DateTime, Utc}; // For TIMESTAMP WITH TIME ZONE

// --- Custom Enum Definitions (Mirroring PostgreSQL ENUMs) ---

#[derive(Debug, PartialEq, Clone, Copy)] // Added Copy
pub enum OrderSide {
    Buy,
    Sell,
    // Add other values as defined in your 'order_side_enum'
}

impl OrderSide {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "BUY" => Ok(OrderSide::Buy),
            "SELL" => Ok(OrderSide::Sell),
            _ => Err(format!("Invalid order_side_enum value: {}", s)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)] // Added Copy
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
    MarketOnClose,
    LimitOnClose,
    Pegged,
    Iceberg,
    Twap,
    Vwap,
    Other,
}

impl OrderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderType::Market => "MARKET",
            OrderType::Limit => "LIMIT",
            OrderType::Stop => "STOP",
            OrderType::StopLimit => "STOP_LIMIT",
            OrderType::MarketOnClose => "MARKET_ON_CLOSE",
            OrderType::LimitOnClose => "LIMIT_ON_CLOSE",
            OrderType::Pegged => "PEGGED",
            OrderType::Iceberg => "ICEBERG",
            OrderType::Twap => "TWAP",
            OrderType::Vwap => "VWAP",
            OrderType::Other => "OTHER",
        }
    }
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "MARKET" => Ok(OrderType::Market),
            "LIMIT" => Ok(OrderType::Limit),
            "STOP" => Ok(OrderType::Stop),
            "STOP_LIMIT" => Ok(OrderType::StopLimit),
            "MARKET_ON_CLOSE" => Ok(OrderType::MarketOnClose),
            "LIMIT_ON_CLOSE" => Ok(OrderType::LimitOnClose),
            "PEGGED" => Ok(OrderType::Pegged),
            "ICEBERG" => Ok(OrderType::Iceberg),
            "TWAP" => Ok(OrderType::Twap),
            "VWAP" => Ok(OrderType::Vwap),
            "OTHER" => Ok(OrderType::Other),
            _ => Err(format!("Invalid order_type_enum value: {}", s)),
        }
    }
}


#[derive(Debug, PartialEq, Clone, Copy)] // Added Copy
pub enum TimeInForce {
    Day,
    Gtc,
    Ioc,
    Fok,
    // Add other values from 'time_in_force_enum'
}

impl TimeInForce {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeInForce::Day => "DAY",
            TimeInForce::Gtc => "GTC",
            TimeInForce::Ioc => "IOC",
            TimeInForce::Fok => "FOK",
        }
    }
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "DAY" => Ok(TimeInForce::Day),
            "GTC" => Ok(TimeInForce::Gtc),
            "IOC" => Ok(TimeInForce::Ioc),
            "FOK" => Ok(TimeInForce::Fok),
            _ => Err(format!("Invalid time_in_force_enum value: {}", s)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)] // Added Copy
pub enum OrderStatus {
    New,              // FIX 0
    PartialFill,      // FIX 1 (PARTIAL_FILL)
    Fill,             // FIX 2 (FILL)
    DoneForDay,       // FIX 3
    Canceled,         // FIX 4
    Replaced,         // FIX 5
    PendingCancel,    // FIX 6
    Stopped,          // FIX 7
    Rejected,         // FIX 8
    Suspended,        // FIX 9
    PendingNew,       // FIX A
    Calculated,       // FIX B
    Expired,          // FIX C
    Restated,         // FIX D
    PendingReplace,   // FIX E
    Trade,            // FIX F
    TradeCorrect,     // FIX G
    PendingValidation, // Custom
    Accepted,          // Custom
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::New => "NEW",
            OrderStatus::PartialFill => "PARTIAL_FILL",
            OrderStatus::Fill => "FILL",
            OrderStatus::DoneForDay => "DONE_FOR_DAY",
            OrderStatus::Canceled => "CANCELED",
            OrderStatus::Replaced => "REPLACED",
            OrderStatus::PendingCancel => "PENDING_CANCEL",
            OrderStatus::Stopped => "STOPPED",
            OrderStatus::Rejected => "REJECTED",
            OrderStatus::Suspended => "SUSPENDED",
            OrderStatus::PendingNew => "PENDING_NEW",
            OrderStatus::Calculated => "CALCULATED",
            OrderStatus::Expired => "EXPIRED",
            OrderStatus::Restated => "RESTATED",
            OrderStatus::PendingReplace => "PENDING_REPLACE",
            OrderStatus::Trade => "TRADE",
            OrderStatus::TradeCorrect => "TRADE_CORRECT",
            OrderStatus::PendingValidation => "PENDING_VALIDATION",
            OrderStatus::Accepted => "ACCEPTED",
        }
    }
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "NEW" => Ok(OrderStatus::New),
            "PARTIAL_FILL" | "PARTIAL FILL" => Ok(OrderStatus::PartialFill),
            "FILL" => Ok(OrderStatus::Fill),
            "DONE_FOR_DAY" | "DONE FOR DAY" => Ok(OrderStatus::DoneForDay),
            "CANCELED" => Ok(OrderStatus::Canceled),
            "REPLACED" => Ok(OrderStatus::Replaced),
            "PENDING_CANCEL" | "PENDING CANCEL" => Ok(OrderStatus::PendingCancel),
            "STOPPED" => Ok(OrderStatus::Stopped),
            "REJECTED" => Ok(OrderStatus::Rejected),
            "SUSPENDED" => Ok(OrderStatus::Suspended),
            "PENDING_NEW" | "PENDING NEW" => Ok(OrderStatus::PendingNew),
            "CALCULATED" => Ok(OrderStatus::Calculated),
            "EXPIRED" => Ok(OrderStatus::Expired),
            "RESTATED" => Ok(OrderStatus::Restated),
            "PENDING_REPLACE" | "PENDING REPLACE" => Ok(OrderStatus::PendingReplace),
            "TRADE" => Ok(OrderStatus::Trade),
            "TRADE_CORRECT" | "TRADE CORRECT" => Ok(OrderStatus::TradeCorrect),
            "PENDING_VALIDATION" => Ok(OrderStatus::PendingValidation),
            "ACCEPTED" => Ok(OrderStatus::Accepted),
            _ => Err(format!("Invalid order_status_enum value: {}", s)),
        }
    }
}


// --- Order Struct ---
#[derive(Debug, Clone)]
pub struct Order {
    pub order_id: Uuid,
    pub client_order_id: String,
    pub original_client_order_id: Option<String>,
    pub exchange_order_id: Option<String>,
    pub account_id: Uuid,
    pub instrument_id: Uuid,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub order_quantity: Decimal,
    pub leaves_quantity: Decimal,
    pub cum_quantity: Decimal,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub currency_code: String,
    pub time_in_force: TimeInForce,
    pub transact_time: DateTime<Utc>,
    pub effective_time: Option<DateTime<Utc>>,
    pub expire_time: Option<DateTime<Utc>>,
    pub order_status: OrderStatus,
    pub execution_destination: Option<String>,
    pub handling_instructions: Option<String>,
    pub text: Option<String>,
    pub last_updated_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    fn from_row(row: &PostgresRow) -> Result<Self, String> {
        Ok(Order {
            order_id: row.try_get("order_id").map_err(|e| e.to_string())?,
            client_order_id: row.try_get("client_order_id").map_err(|e| e.to_string())?,
            original_client_order_id: row.try_get("original_client_order_id").ok(),
            exchange_order_id: row.try_get("exchange_order_id").ok(),
            account_id: row.try_get("account_id").map_err(|e| e.to_string())?,
            instrument_id: row.try_get("instrument_id").map_err(|e| e.to_string())?,
            side: OrderSide::from_str(row.try_get("side").map_err(|e| e.to_string())?)?,
            order_type: OrderType::from_str(row.try_get("order_type").map_err(|e| e.to_string())?)?,
            order_quantity: row.try_get("order_quantity").map_err(|e| e.to_string())?,
            leaves_quantity: row.try_get("leaves_quantity").map_err(|e| e.to_string())?,
            cum_quantity: row.try_get("cum_quantity").map_err(|e| e.to_string())?,
            price: row.try_get("price").ok(),
            stop_price: row.try_get("stop_price").ok(),
            currency_code: row.try_get("currency_code").map_err(|e| e.to_string())?,
            time_in_force: TimeInForce::from_str(row.try_get("time_in_force").map_err(|e| e.to_string())?)?,
            transact_time: row.try_get("transact_time").map_err(|e| e.to_string())?,
            effective_time: row.try_get("effective_time").ok(),
            expire_time: row.try_get("expire_time").ok(),
            order_status: OrderStatus::from_str(row.try_get("order_status").map_err(|e| e.to_string())?)?,
            execution_destination: row.try_get("execution_destination").ok(),
            handling_instructions: row.try_get("handling_instructions").ok(),
            text: row.try_get("text").ok(),
            last_updated_by: row.try_get("last_updated_by").ok(),
            created_at: row.try_get("created_at").map_err(|e| e.to_string())?,
            updated_at: row.try_get("updated_at").map_err(|e| e.to_string())?,
        })
    }
}

// --- Database Connection Pool ---
pub type DbPool = r2d2::Pool<PostgresConnectionManager<NoTls>>;

pub fn create_db_pool(db_url: &str) -> Result<DbPool, String> { // Return String error for simplicity here
    let pg_config = PgConfig::from_str(db_url)
        .map_err(|e| format!("Failed to parse database URL '{}': {}", db_url, e))?;

    let manager = PostgresConnectionManager::new(
        pg_config,
        NoTls,
    );
    r2d2::Pool::builder()
        .build(manager)
        .map_err(|e| format!("Failed to build R2D2 connection pool: {}", e))
}


// --- Database Operations ---

#[derive(Debug)]
pub enum DbError {
    Postgres(PostgresError),
    R2D2(r2d2::Error),
    Mapping(String),
    NotFound,
    UpdateFailed(String),
}

impl From<PostgresError> for DbError {
    fn from(err: PostgresError) -> DbError {
        DbError::Postgres(err)
    }
}

impl From<r2d2::Error> for DbError {
    fn from(err: r2d2::Error) -> DbError {
        DbError::R2D2(err)
    }
}
impl From<String> for DbError { // For Order::from_row mapping errors & create_db_pool
    fn from(err: String) -> DbError {
        DbError::Mapping(err)
    }
}


pub struct NewOrderRequest {
    pub client_order_id: String,
    pub account_id: Uuid,
    pub instrument_id: Uuid,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub order_quantity: Decimal,
    pub price: Option<Decimal>,
    pub currency_code: String,
    pub time_in_force: Option<TimeInForce>,
    pub execution_destination: Option<String>,
    pub text: Option<String>,
}

pub async fn create_order(
    pool: &DbPool,
    order_data: &NewOrderRequest,
    initial_status: OrderStatus,
) -> Result<Uuid, DbError> {
    let mut client = pool.get()?;

    let order_id = Uuid::new_v4();
    let transact_time = Utc::now();
    let leaves_quantity = order_data.order_quantity;
    let cum_quantity = Decimal::ZERO;
    // Safely get time_in_force, defaulting to Day if None
    let time_in_force_val = order_data.time_in_force.unwrap_or(TimeInForce::Day);

    let row = client.query_one(
        r#"
        INSERT INTO orders (
            order_id, client_order_id, account_id, instrument_id, side,
            order_type, order_quantity, leaves_quantity, cum_quantity, price,
            currency_code, time_in_force, transact_time, order_status,
            execution_destination, text
        ) VALUES (
            $1, $2, $3, $4, $5::order_side_enum,
            $6::order_type_enum, $7, $8, $9, $10,
            $11, $12::time_in_force_enum, $13, $14::order_status_enum,
            $15, $16
        )
        RETURNING order_id
        "#,
        &[
            &order_id,
            &order_data.client_order_id,
            &order_data.account_id,
            &order_data.instrument_id,
            &order_data.side.as_str(),
            &order_data.order_type.as_str(),
            &order_data.order_quantity,
            &leaves_quantity, 
            &cum_quantity,    
            &order_data.price,
            &order_data.currency_code,
            &time_in_force_val.as_str(), // Use the unwrapped or default value
            &transact_time,
            &initial_status.as_str(),
            &order_data.execution_destination,
            &order_data.text,
        ],
    )?;

    Ok(row.get(0))
}

pub async fn get_order_by_id(pool: &DbPool, order_id_to_find: Uuid) -> Result<Option<Order>, DbError> {
    let mut client = pool.get()?;

    let row_opt = client.query_opt(
        r#"
        SELECT
            order_id, client_order_id, original_client_order_id, exchange_order_id,
            account_id, instrument_id, side, order_type, order_quantity,
            leaves_quantity, cum_quantity, price, stop_price, currency_code,
            time_in_force, transact_time, effective_time, expire_time,
            order_status, execution_destination, handling_instructions, text,
            last_updated_by, created_at, updated_at
        FROM orders
        WHERE order_id = $1
        "#,
        &[&order_id_to_find],
    )?;

    match row_opt {
        Some(row) => Ok(Some(Order::from_row(&row)?)),
        None => Ok(None),
    }
}

pub async fn update_order_status(
    pool: &DbPool,
    order_id_to_update: Uuid,
    new_status: OrderStatus,
    // last_updated_by: Option<&str>, // Consider adding who updated it
) -> Result<u64, DbError> {
    let mut client = pool.get()?;
    let updated_at_time = Utc::now();

    let rows_affected = client.execute(
        r#"
        UPDATE orders
        SET 
            order_status = $1::order_status_enum,
            updated_at = $2
        WHERE order_id = $3
        "#,
        &[
            &new_status.as_str(),
            &updated_at_time,
            &order_id_to_update,
            // &last_updated_by, // If you add this parameter
        ],
    )?;

    if rows_affected == 0 {
        Err(DbError::NotFound) // Or a more specific UpdateFailedNoRows error
    } else {
        Ok(rows_affected)
    }
}


// TODO:
// - Add functions for updating other order fields (e.g. price, quantity for replace requests).
// - Ensure Cargo.toml has `postgres` (with "with-chrono-0_4", "with-rust_decimal-1", "with-uuid-1" features),
//   `r2d2`, `r2d2-postgres`, `uuid` (with "v4" feature), `rust_decimal` (with "db-postgres"), `chrono` (with "serde"). (DONE for basic setup)
// - Add robust error handling for db_url.parse() in create_db_pool. (IMPROVED)
// - Consider using a macro for cleaner enum from_str/as_str implementations (e.g., `strum` crate).
// - Add `last_updated_by` to `update_order_status` and relevant table columns if needed.
