// TODO Complete verification of test results

use crate::ai::dto::{
    EmpericalObservation, EmpericalObservationColumn, FunctionalRoleAnalysisResult,
    SchemaLinkingFinalSynthesisResponse, SchemaLinkingRefinedTable, SchemaLinkingRejectedCandidate,
    SchemaLinkingRelevantColumn, TableFunction,
};
use crate::ai::text_to_sql_agent::{
    execute, execute_hypothesis_verification, execute_search_space_reduction,
    schema_linking_final_synthesis, sql_generation,
};
use crate::common::{AppState, Configurations};
use crate::connection::constants::SourceType;
use crate::connection::dto::CreateConnectionDto;
use crate::connection::service::{execute_create_connection, execute_delete_connection};
use crate::data_catalog::dto::{Column, Database, Schema, Table};
use crate::{db, entity};
use rig::providers::anthropic;
use std::path::Path;
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

const TEST_DATABASE_PATH: &str = "./data/integration_test.db";

async fn setup_test_env() -> (AppState, entity::connection::Model) {
    let app_state = init_app_state().await;
    let connection = create_test_connection(&app_state).await;
    (app_state, connection)
}

async fn teardown_test_env(app_state: &AppState, connection_id: Uuid) {
    let _ = execute_delete_connection(app_state, connection_id).await;
    if Path::new(TEST_DATABASE_PATH).exists() {
        let _ = std::fs::remove_file(TEST_DATABASE_PATH);
    }
}

async fn init_app_state() -> AppState {
    let database_url = format!("sqlite://{}?mode=rwc", TEST_DATABASE_PATH);
    if let Some(parent) = Path::new(TEST_DATABASE_PATH).parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let database_connection = db::get_db(&database_url).await.unwrap();
    db::run_migrations(&database_connection).await.unwrap();
    let config = Configurations::builder()
        .database_url(database_url.clone())
        .encryption_key("test-secret-key-32-chars-long!!")
        .admin_username("admin")
        .admin_password("test_password")
        .admin_email("admin@test.local")
        .admin_full_name("Test Admin")
        .build()
        .unwrap();
    let client = reqwest::Client::new();
    AppState::new(database_connection, config, client)
}

async fn create_test_connection(app_state: &AppState) -> entity::connection::Model {
    match execute_create_connection(
        app_state,
        CreateConnectionDto {
            name: "Ecommerce Sales Demo".to_string(),
            username: None,
            password: None,
            host: None,
            port: None,
            database: format!("{}/demo/ecommerce.db", env!("CARGO_MANIFEST_DIR")),
            schema: None,
            description: Some("Demo SQLite database with ecommerce sales data".to_string()),
            source_type: SourceType::Sqlite,
        },
    )
    .await
    {
        Ok(c) => return c,
        Err(err) => {
            error!("Failed to create test connection: {}", err);
            panic!("Failed to create test connection");
        }
    };
}

fn test_create_client() -> anthropic::Client {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY must be set to run these tests");
    anthropic::Client::new(&api_key).unwrap()
}

fn sample_pruned_data_catalog(connection_name: &str, connection_description: &str) -> Database {
    Database::new(
        connection_name,
        connection_description,
        vec![Schema::new(
            "main",
            "",
            vec![
                Table::new(
                    "sellers",
                    "",
                    vec![(
                        "seller_id",
                        Column::with_samples(
                            "seller_id",
                            "",
                            [
                                "0015a82c2db000af6aaaf3ae2ecb0532",
                                "001cca7ae9ae17fb1caed9dfb1094831",
                                "002100f778ceb8431b7a1020ff7ab48f",
                                "003554e2dce176b5555353e4f3555ac8",
                                "004c9cd9d87a3c30c522c48c4fc07416",
                            ],
                            "TEXT",
                            true,
                            false,
                        ),
                    )],
                    false,
                    false,
                ),
                Table::new(
                    "order_items",
                    "",
                    vec![
                        (
                            "order_id",
                            Column::with_samples(
                                "order_id",
                                "",
                                [
                                    "001d8f0e34a38c37f7dba2a37d4eba8b",
                                    "0020262c8a370bd5a174ea6a2a267321",
                                    "00259a44fcad3fc0474329e925d14fc3",
                                    "00335b686d693c7d72deeb12f8e89227",
                                    "00337fe25a3780b3424d9ad7c5a4b35e",
                                ],
                                "TEXT",
                                true,
                                false,
                            ),
                        ),
                        (
                            "freight_value",
                            Column::with_samples(
                                "freight_value",
                                "",
                                ["7.78", "21.05", "14.1", "16.89", "9.94"],
                                "REAL",
                                true,
                                false,
                            ),
                        ),
                        (
                            "price",
                            Column::with_samples(
                                "price",
                                "",
                                ["18.99", "79.5", "19.99", "63.9", "59.9"],
                                "REAL",
                                true,
                                false,
                            ),
                        ),
                    ],
                    false,
                    false,
                ),
                Table::new(
                    "order_payments",
                    "",
                    vec![
                        (
                            "order_id",
                            Column::with_samples(
                                "order_id",
                                "",
                                [
                                    "0010dedd556712d7bb69a19cb7bbd37a",
                                    "001d8f0e34a38c37f7dba2a37d4eba8b",
                                    "0020262c8a370bd5a174ea6a2a267321",
                                    "00259a44fcad3fc0474329e925d14fc3",
                                    "00335b686d693c7d72deeb12f8e89227",
                                ],
                                "TEXT",
                                true,
                                false,
                            ),
                        ),
                        (
                            "payment_value",
                            Column::with_samples(
                                "payment_value",
                                "",
                                ["51.84", "105.28", "170.57", "173.84", "47.72"],
                                "REAL",
                                true,
                                false,
                            ),
                        ),
                    ],
                    false,
                    false,
                ),
                Table::new(
                    "order_reviews",
                    "",
                    vec![
                        (
                            "review_id",
                            Column::with_samples(
                                "review_id",
                                "",
                                [
                                    "000a4fc2877b9726aecc081ad467bbf4",
                                    "000d524a3c693e342d163912ad74f156",
                                    "000faedabef50689a0ea3fc4fbf99cb4",
                                    "0012aca79505926b2587d1140c08fad6",
                                    "001b3b4666d0e109d7563e1465f4ba60",
                                ],
                                "TEXT",
                                true,
                                false,
                            ),
                        ),
                        (
                            "order_id",
                            Column::with_samples(
                                "order_id",
                                "",
                                [
                                    "0010dedd556712d7bb69a19cb7bbd37a",
                                    "001d8f0e34a38c37f7dba2a37d4eba8b",
                                    "0020262c8a370bd5a174ea6a2a267321",
                                    "00259a44fcad3fc0474329e925d14fc3",
                                    "00337fe25a3780b3424d9ad7c5a4b35e",
                                ],
                                "TEXT",
                                true,
                                false,
                            ),
                        ),
                    ],
                    false,
                    false,
                ),
                Table::new(
                    "customers",
                    "",
                    vec![(
                        "customer_id",
                        Column::with_samples(
                            "customer_id",
                            "",
                            [
                                "00072d033fe2e59061ae5c3aff1a2be5",
                                "000e943451fc2788ca6ac98a682f2f49",
                                "0010068a73b7c56da5758c3f9e5c7375",
                                "00114026c1b7b52ab1773f317ef4880b",
                                "00155f0530cc7b2bf73cc3f81cb01c52",
                            ],
                            "TEXT",
                            true,
                            false,
                        ),
                    )],
                    false,
                    false,
                ),
                Table::new(
                    "products",
                    "",
                    vec![(
                        "product_id",
                        Column::with_samples(
                            "product_id",
                            "",
                            [
                                "001795ec6f1b187d37335e1c4704762e",
                                "001b72dfd63e9833e8c02742adf472e3",
                                "0021a87d4997a48b6cef1665602be0f5",
                                "003dbcabcf8e3231de657c7d9f9a5eba",
                                "005030ef108f58b46b78116f754d8d38",
                            ],
                            "TEXT",
                            true,
                            false,
                        ),
                    )],
                    false,
                    false,
                ),
                Table::new(
                    "orders",
                    "",
                    vec![
                        (
                            "customer_id",
                            Column::with_samples(
                                "customer_id",
                                "",
                                [
                                    "00072d033fe2e59061ae5c3aff1a2be5",
                                    "000e943451fc2788ca6ac98a682f2f49",
                                    "0010068a73b7c56da5758c3f9e5c7375",
                                    "00114026c1b7b52ab1773f317ef4880b",
                                    "00155f0530cc7b2bf73cc3f81cb01c52",
                                ],
                                "TEXT",
                                true,
                                false,
                            ),
                        ),
                        (
                            "order_id",
                            Column::with_samples(
                                "order_id",
                                "",
                                [
                                    "0010dedd556712d7bb69a19cb7bbd37a",
                                    "001d8f0e34a38c37f7dba2a37d4eba8b",
                                    "0020262c8a370bd5a174ea6a2a267321",
                                    "00259a44fcad3fc0474329e925d14fc3",
                                    "00335b686d693c7d72deeb12f8e89227",
                                ],
                                "TEXT",
                                true,
                                false,
                            ),
                        ),
                    ],
                    false,
                    false,
                ),
            ],
            false,
            false,
        )],
    )
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,rig=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::test]
#[ignore]
async fn execute_works() {
    init_tracing();
    let client = test_create_client();
    let (app_state, connection) = setup_test_env().await;
    let sql = execute(
        &app_state,
        &connection,
        "What are the top 5 customers by revenue?",
        &client,
    )
    .await
    .unwrap();
    teardown_test_env(&app_state, connection.id).await;
}

#[tokio::test]
#[ignore]
async fn execute_search_space_reduction_works() {
    init_tracing();
    let (app_state, connection) = setup_test_env().await;
    let client = test_create_client();
    let pruned_data_catalog = execute_search_space_reduction(
        &app_state,
        &connection,
        "What are the top 5 customers by revenue?",
        MASTER_PLAN_TOP_5_CUSTOMERS,
        &client,
    )
    .await
    .unwrap();
    teardown_test_env(&app_state, connection.id).await;
}

#[tokio::test]
#[ignore]
async fn execute_hypothesis_verification_works() {
    init_tracing();
    let (app_state, connection) = setup_test_env().await;
    let client = test_create_client();
    let pruned_data_catalog = sample_pruned_data_catalog(
        &connection.name,
        connection.description.as_deref().unwrap_or(""),
    );
    execute_hypothesis_verification(
        &app_state,
        &connection,
        "What are the top 5 customers by revenue?",
        r#"
            1. Identify all customer entities in the system
            2. Link each customer to their associated transactions or orders
            3. Calculate the total revenue generated by each customer
            4. Sort customers in descending order by their calculated total revenue
            5. Select the top 5 customers from the sorted list
        "#,
        &pruned_data_catalog,
        &client,
    )
    .await
    .unwrap();

    teardown_test_env(&app_state, connection.id).await;
}

fn sample_functional_role_analysis_result() -> FunctionalRoleAnalysisResult {
    FunctionalRoleAnalysisResult {
        database_structure: "The Ecommerce Sales Demo is a SQLite database modeling an e-commerce platform with the following key entities: (1) customers - stores customer identifiers; (2) orders - links customers to their orders via customer_id and order_id; (3) order_items - contains line-item details including price and freight_value, linked to orders via order_id; (4) order_payments - records payment transactions linked to orders via order_id with payment_value; (5) order_reviews - stores customer reviews linked to orders via order_id; (6) products - product catalog with product_id; (7) sellers - seller information with seller_id. The primary join path for revenue calculation flows through: customers → orders (via customer_id) → order_items (via order_id) and/or order_payments (via order_id).".to_string(),
        query_specific_content_analysis: "To identify the top 5 customers by revenue, the query requires: (1) Customer identification from the customers table (customer_id); (2) Revenue aggregation which can be calculated from order_items.price (product revenue) + order_items.freight_value (shipping revenue), or alternatively from order_payments.payment_value (total payment received); (3) Linking customers to revenue via the orders table which serves as the bridge connecting customers to their transactional data; (4) Aggregation and sorting by total revenue in descending order; (5) Limiting results to top 5. The most semantically accurate revenue metric would be order_payments.payment_value as it represents actual revenue received, though order_items.price + order_items.freight_value provides an alternative calculation method. The orders table is critical as it contains the customer_id foreign key that links each order to a specific customer.".to_string(),
        table_functions: vec![
            TableFunction {
                database: "Ecommerce Sales Demo".to_string(),
                schema: "main".to_string(),
                table: "customers".to_string(),
                table_function: "Target table containing customer_id, the primary entity dimension needed to identify and display the top 5 customers in the result set.".to_string(),
            },
            TableFunction {
                database: "Ecommerce Sales Demo".to_string(),
                schema: "main".to_string(),
                table: "orders".to_string(),
                table_function: "Bridge table connecting customers to their transactional data via customer_id (foreign key to customers) and order_id (foreign key to order_items and order_payments). Essential for linking each customer to their associated orders.".to_string(),
            },
            TableFunction {
                database: "Ecommerce Sales Demo".to_string(),
                schema: "main".to_string(),
                table: "order_items".to_string(),
                table_function: "Target table for revenue calculation, containing price and freight_value columns that can be summed to compute total revenue per customer when aggregated through orders.".to_string(),
            },
            TableFunction {
                database: "Ecommerce Sales Demo".to_string(),
                schema: "main".to_string(),
                table: "order_payments".to_string(),
                table_function: "Target table for revenue calculation, containing payment_value which represents actual revenue received per order. Linked to orders via order_id and provides an alternative/complementary revenue metric to order_items pricing.".to_string(),
            },
        ],
    }
}

fn sample_emperical_observations() -> Vec<EmpericalObservation> {
    vec![
        EmpericalObservation {
            relevant: true,
            relevant_columns: vec![EmpericalObservationColumn {
                column_name: "customer_id".to_string(),
                relevance_reason: "Provides the primary identifier needed to display and rank the top 5 customers in the result set. This is the entity dimension required to answer 'top 5 customers by revenue'.".to_string(),
                observations: "Column contains 10,000 unique customer identifiers with no NULL values. Each customer_id is unique (one row per customer), verified by COUNT(DISTINCT) matching total row count. Data type is TEXT with hash-like values.".to_string(),
            }],
            table_summary: "The customers table is a customer dimension table containing 10,000 unique customers identified by customer_id, along with demographic attributes (city, state, zip code). It serves as the customer master record and provides the customer identifiers needed to identify which customers to display in the top 5 revenue ranking. However, this table does NOT contain revenue data itself—revenue information must be sourced from a separate transactions, orders, or sales table and joined to this table using customer_id as the bridge key.".to_string(),
        },
        EmpericalObservation {
            relevant: true,
            relevant_columns: vec![
                EmpericalObservationColumn {
                    column_name: "customer_id".to_string(),
                    relevance_reason: "Essential bridge key to group orders by customer for revenue aggregation. Enables the GROUP BY operation required to identify top 5 customers.".to_string(),
                    observations: "Contains 10,000 distinct customer identifiers with zero NULL values. All values are populated and non-null, enabling reliable joins and grouping.".to_string(),
                },
                EmpericalObservationColumn {
                    column_name: "order_id".to_string(),
                    relevance_reason: "Provides the order-level grain and serves as foreign key to join with order_items and order_payments tables where revenue amounts are stored. Critical for linking to transactional revenue data.".to_string(),
                    observations: "Contains 10,000 distinct order identifiers with zero NULL values. One-to-one relationship with customer_id (10,000 rows, 10,000 distinct orders). No duplicate order_ids exist in the table.".to_string(),
                },
            ],
            table_summary: "The orders table is a bridge table with one row per order, containing 10,000 orders from 10,000 customers. It serves as the essential linking mechanism between customers and their transactional data via customer_id and order_id foreign keys. While this table does not contain revenue amounts directly, it provides the necessary keys to join with order_items and order_payments tables to calculate customer revenue. The table is fully populated with no NULL values in critical columns, making it a reliable bridge for answering the top 5 customers by revenue question.".to_string(),
        },
        EmpericalObservation {
            relevant: true,
            relevant_columns: vec![
                EmpericalObservationColumn {
                    column_name: "order_id".to_string(),
                    relevance_reason: "Provides the bridge/join key needed to connect order items to customer data through the orders table, enabling customer-level revenue aggregation.".to_string(),
                    observations: "Contains 11,335 non-null values across 9,890 distinct orders. All values are populated (no NULLs). This is a valid foreign key that links items to orders.".to_string(),
                },
                EmpericalObservationColumn {
                    column_name: "price".to_string(),
                    relevance_reason: "Primary numerical column for revenue calculation. Must be summed per customer to compute total revenue for ranking top 5 customers.".to_string(),
                    observations: "Contains 11,335 non-null values with no missing data. Range: $2.29 to $3,980.00. No negative values detected. Data quality is excellent.".to_string(),
                },
                EmpericalObservationColumn {
                    column_name: "freight_value".to_string(),
                    relevance_reason: "Secondary numerical column for revenue calculation. Must be summed alongside price per customer to compute total revenue for ranking top 5 customers.".to_string(),
                    observations: "Contains 11,335 non-null values with no missing data. Range: $0.00 to $314.02. No negative values detected. Data quality is excellent.".to_string(),
                },
            ],
            table_summary: "The order_items table is a line-item detail table containing 11,335 rows representing individual items within orders. Each row represents one item in an order (grain: one row per item per order). The table contains price and freight_value columns that can be summed to calculate total revenue per order. When joined to an orders table via order_id and then aggregated by customer, this table directly supports the revenue calculation needed to identify the top 5 customers by revenue. All critical columns are fully populated with valid numeric data suitable for aggregation.".to_string(),
        },
        EmpericalObservation {
            relevant: false,
            relevant_columns: vec![
                EmpericalObservationColumn {
                    column_name: "order_id".to_string(),
                    relevance_reason: "Provides a bridge key to orders, but insufficient alone to answer the customer revenue question without access to customer identifiers.".to_string(),
                    observations: "Column contains 10,000 distinct order IDs with no NULL values. However, this table lacks a customer_id column, making it impossible to directly link payments to customers without joining to another table (orders or customers table).".to_string(),
                },
                EmpericalObservationColumn {
                    column_name: "payment_value".to_string(),
                    relevance_reason: "Provides the revenue metric needed for aggregation, but the granularity (multiple payments per order) requires careful aggregation logic.".to_string(),
                    observations: "Contains 10,574 payment records with values ranging from $0.00 to $4,042.74 (average $155.60). Only 2 non-positive values exist. Multiple payment records per order are common (574 orders have multiple payments), indicating this table tracks payment installments or partial payments rather than final order totals.".to_string(),
                },
            ],
            table_summary: "The order_payments table contains payment transaction records with order IDs and payment values, but critically lacks a customer_id column. While it provides revenue data via payment_value, it cannot directly answer 'top 5 customers by revenue' without joining to an orders or customers table to establish the customer-order relationship. The table represents payment installments/transactions rather than complete order summaries, requiring aggregation at the order level before customer-level analysis is possible.".to_string(),
        },
    ]
}

fn sample_linked_schema() -> SchemaLinkingFinalSynthesisResponse {
    SchemaLinkingFinalSynthesisResponse {
        exploration_queries: vec![],
        refined_schema: vec![
            SchemaLinkingRefinedTable {
                table_name: "customers".to_string(),
                relevant_columns: vec![SchemaLinkingRelevantColumn {
                    column_name: "customer_id".to_string(),
                    relevance_reason: "Identification: Primary identifier needed to display and rank the top 5 customers in the result set. This is the entity dimension required to answer 'top 5 customers by revenue'.".to_string(),
                }],
            },
            SchemaLinkingRefinedTable {
                table_name: "orders".to_string(),
                relevant_columns: vec![
                    SchemaLinkingRelevantColumn {
                        column_name: "customer_id".to_string(),
                        relevance_reason: "Linking: Foreign key to customers table. Essential bridge key to group orders by customer for revenue aggregation and to enable the GROUP BY operation required to identify top 5 customers.".to_string(),
                    },
                    SchemaLinkingRelevantColumn {
                        column_name: "order_id".to_string(),
                        relevance_reason: "Linking: Foreign key to order_items and order_payments tables. Provides the order-level grain and serves as the join key to connect to transactional revenue data where revenue amounts are stored.".to_string(),
                    },
                ],
            },
            SchemaLinkingRefinedTable {
                table_name: "order_items".to_string(),
                relevant_columns: vec![
                    SchemaLinkingRelevantColumn {
                        column_name: "order_id".to_string(),
                        relevance_reason: "Linking: Foreign key to orders table. Provides the bridge/join key needed to connect order items to customer data through the orders table, enabling customer-level revenue aggregation.".to_string(),
                    },
                    SchemaLinkingRelevantColumn {
                        column_name: "price".to_string(),
                        relevance_reason: "Aggregation: Primary numerical column for revenue calculation. Must be summed per customer to compute total revenue for ranking top 5 customers.".to_string(),
                    },
                    SchemaLinkingRelevantColumn {
                        column_name: "freight_value".to_string(),
                        relevance_reason: "Aggregation: Secondary numerical column for revenue calculation. Must be summed alongside price per customer to compute total revenue for ranking top 5 customers.".to_string(),
                    },
                ],
            },
        ],
        rejected_candidates: vec![
            SchemaLinkingRejectedCandidate {
                table: "order_payments".to_string(),
                column: "order_id".to_string(),
                reject_reason: "Originally marked IRRELEVANT in order_payments table. While it is a valid foreign key to orders, the order_payments table is not needed because order_items (price + freight_value) provides the same revenue calculation with cleaner aggregation logic (one row per item). Both approaches yield identical top 5 customer rankings, making order_payments redundant.".to_string(),
            },
            SchemaLinkingRejectedCandidate {
                table: "order_payments".to_string(),
                column: "payment_value".to_string(),
                reject_reason: "Originally marked IRRELEVANT in order_payments table. While it represents revenue, the order_payments table has multiple payment records per order (installments/partial payments), requiring complex aggregation. The order_items table (price + freight_value) provides equivalent revenue calculation with simpler, more direct aggregation logic. Both methods produce identical results for top 5 customers.".to_string(),
            },
        ],
        status: "CONFIRM".to_string(),
    }
}

const MASTER_PLAN_TOP_5_CUSTOMERS: &str = r#"
            1. Identify all customer entities in the system
            2. Link each customer to their associated transactions or orders
            3. Calculate the total revenue generated by each customer by summing all transaction amounts or order values
            4. Sort customers in descending order by their calculated total revenue
            5. Select the top 5 customers from the sorted list
            6. Return customer identifiers (and relevant customer details) along with their calculated total revenue
        "#;

#[tokio::test]
#[ignore]
async fn schema_linking_final_synthesis_works() {
    init_tracing();
    let (app_state, connection) = setup_test_env().await;
    let client = test_create_client();
    let pruned_data_catalog = sample_pruned_data_catalog(
        &connection.name,
        connection.description.as_deref().unwrap_or(""),
    );
    let result = schema_linking_final_synthesis(
        &connection,
        "What are the top 5 customers by revenue?",
        sample_functional_role_analysis_result(),
        &sample_emperical_observations(),
        &pruned_data_catalog,
        &client,
    )
    .await
    .unwrap();
    teardown_test_env(&app_state, connection.id).await;
}

#[tokio::test]
#[ignore]
async fn sql_generation_works() {
    init_tracing();
    let (app_state, connection) = setup_test_env().await;
    let client = test_create_client();
    let result = sql_generation(
        &connection,
        &client,
        "What are the top 5 customers by revenue?",
        MASTER_PLAN_TOP_5_CUSTOMERS,
        sample_linked_schema(),
    )
    .await
    .unwrap();
    teardown_test_env(&app_state, connection.id).await;
}
