use crate::support::TestApp;
use crate::support::canvas_builder::string_field;
use crate::support::constants::{
    ORDER_STATUS_QUESTION, POSTGRES_SAMPLE_CONNECTION_NAME, SAMPLE_DATABASE_SCHEMA,
    SQLITE_DEMO_CONNECTION_NAME,
};
use crate::support::conversation::{event_data, event_names, run_completion, start_conversation};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

async fn introspected_catalog(app: &TestApp, connection: &Value) -> Value {
    let conversation_id =
        start_conversation(app, &string_field(connection, "id"), ORDER_STATUS_QUESTION).await;
    let events = run_completion(app, &conversation_id).await;
    assert_eq!(
        event_names(&events).last().map(String::as_str),
        Some("completed"),
        "{:?}",
        event_names(&events)
    );
    event_data(&events, "executed_search_space_reduction")["pruned_data_catalog"].clone()
}

fn table_names(catalog: &Value, schema: &str) -> BTreeSet<String> {
    catalog["schemas"][schema]["tables"]
        .as_object()
        .unwrap_or_else(|| panic!("schema {schema} is missing from {catalog}"))
        .keys()
        .cloned()
        .collect()
}

fn column_types(catalog: &Value, schema: &str, table: &str) -> BTreeMap<String, String> {
    catalog["schemas"][schema]["tables"][table]["columns"]
        .as_object()
        .unwrap_or_else(|| panic!("table {schema}.{table} is missing from {catalog}"))
        .values()
        .map(|column| {
            (
                string_field(column, "name"),
                string_field(column, "data_type"),
            )
        })
        .collect()
}

fn expected_columns(columns: &[(&str, &str)]) -> BTreeMap<String, String> {
    columns
        .iter()
        .map(|(name, data_type)| (name.to_string(), data_type.to_string()))
        .collect()
}

#[tokio::test]
async fn sqlite_catalog_describes_every_demo_table_and_column() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    app.configure_live_ai_provider("anthropic").await;
    let connection = app.create_sqlite_demo_connection().await;

    let catalog = introspected_catalog(&app, &connection).await;

    assert_eq!(catalog["name"], SQLITE_DEMO_CONNECTION_NAME);
    assert_eq!(
        table_names(&catalog, "main"),
        BTreeSet::from(
            [
                "customers",
                "order_items",
                "order_payments",
                "order_reviews",
                "orders",
                "products",
                "sellers",
            ]
            .map(String::from)
        )
    );
    assert_eq!(
        column_types(&catalog, "main", "orders"),
        expected_columns(&[
            ("order_id", "TEXT"),
            ("customer_id", "TEXT"),
            ("order_status", "TEXT"),
            ("order_purchase_timestamp", "TEXT"),
            ("order_approved_at", "TEXT"),
            ("order_delivered_carrier_date", "TEXT"),
            ("order_delivered_customer_date", "TEXT"),
            ("order_estimated_delivery_date", "TEXT"),
        ])
    );
    assert_eq!(
        column_types(&catalog, "main", "order_items"),
        expected_columns(&[
            ("order_id", "TEXT"),
            ("order_item_id", "INTEGER"),
            ("product_id", "TEXT"),
            ("seller_id", "TEXT"),
            ("shipping_limit_date", "TEXT"),
            ("price", "REAL"),
            ("freight_value", "REAL"),
        ])
    );
}

#[tokio::test]
async fn postgres_catalog_describes_the_sample_tables_and_columns() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    app.configure_live_ai_provider("anthropic").await;
    let connection = app.create_postgres_sample_connection().await;

    let catalog = introspected_catalog(&app, &connection).await;

    assert_eq!(catalog["name"], POSTGRES_SAMPLE_CONNECTION_NAME);
    assert_eq!(
        table_names(&catalog, SAMPLE_DATABASE_SCHEMA),
        BTreeSet::from(["customers", "orders"].map(String::from))
    );
    assert_eq!(
        column_types(&catalog, SAMPLE_DATABASE_SCHEMA, "customers"),
        expected_columns(&[("id", "integer"), ("name", "text"), ("country", "text")])
    );
    assert_eq!(
        column_types(&catalog, SAMPLE_DATABASE_SCHEMA, "orders"),
        expected_columns(&[
            ("id", "integer"),
            ("customer_id", "integer"),
            ("order_status", "text"),
            ("amount", "numeric"),
            ("ordered_at", "date"),
        ])
    );
}
