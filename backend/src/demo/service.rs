use std::collections::HashMap;

use tracing::{error, info};
use uuid::Uuid;

use crate::canvas::dto::CreateCanvasDto;
use crate::canvas::service::execute_create_canvas;
use crate::cell::dto::CreateCellDto;
use crate::cell::service::execute_create_cell;
use crate::common::AppState;
use crate::connection::constants::SourceType;
use crate::connection::dto::CreateConnectionDto;
use crate::connection::service::execute_create_connection;
use crate::dashboard::service::{execute_update_dashboard, get_dashboard_by_canvas_id_entity};
use crate::dashboard::{DashboardDto, Layout};

pub async fn seed_demo_data(app_state: &AppState) {
    if crate::configuration::service::is_demo_data_setup_done(app_state).await {
        return;
    }

    let connection = match execute_create_connection(
        app_state,
        CreateConnectionDto {
            name: "Ecommerce Sales Demo".to_string(),
            username: None,
            password: None,
            host: None,
            port: None,
            database: format!("{}/src/demo/ecommerce.db", env!("CARGO_MANIFEST_DIR")),
            schema: None,
            description: Some("Demo SQLite database with ecommerce sales data".to_string()),
            source_type: SourceType::Sqlite,
        },
    )
    .await
    {
        Ok(c) => c,
        Err(err) => {
            error!(
                "Failed to create connection when seeding demo data: {}",
                err
            );
            return;
        }
    };

    let canvas_result = match execute_create_canvas(
        app_state,
        CreateCanvasDto {
            name: "Ecommerce Sales".to_string(),
            description: Some("Ecommerce sales analytics".to_string()),
        },
    )
    .await
    {
        Ok(r) => r,
        Err(err) => {
            error!("Failed to create canvas when seeding demo data: {}", err);
            return;
        }
    };

    let connection_id = connection.id;
    let notebook_id = canvas_result.notebook.id;

    let sql_cells: Vec<(i32, &'static str, &'static str)> = vec![
        (0, "Avg Review Score", "select AVG(review_score) from order_reviews"),
        (2, "Total GMV", "select sum(oi.price + oi.freight_value) from order_items oi inner join orders o on oi.order_id = o.order_id"),
        (4, "On-Time Delivery Rate", "SELECT CAST(SUM(CASE WHEN order_delivered_customer_date <= order_estimated_delivery_date THEN 1 ELSE 0 END) AS FLOAT) * 100 / COUNT(1) as on_time_delivery_percentage FROM orders WHERE order_delivered_customer_date IS NOT NULL"),
        (7, "Monthly Revenue", "select SUM(op.payment_value) as revenue, strftime('%Y-%m', o.order_purchase_timestamp) as order_month from orders o inner join order_payments op on o.order_id = op.order_id group by order_month order by order_month asc"),
        (9, "Orders by Status", "select count(1) as order_count, order_status from orders group by order_status"),
        (11, "Avg Order Value", "select ROUND(AVG(op.payment_value),2) as avg_order_value, strftime('%Y-%m', o.order_purchase_timestamp) as order_month from orders o inner join order_payments op on o.order_id = op.order_id group by order_month"),
        (13, "Top Products", "select p.product_id, SUM(oi.price) as total_revenue from order_items oi inner join products p on oi.product_id = p.product_id group by p.product_id order by total_revenue desc limit 10"),
        (15, "Top Sellers GMV", "select seller_id, SUM(price) as gmv from order_items group by seller_id order by gmv desc limit 10"),
        (17, "Revenue by Payment Type", "select payment_type, SUM(payment_value) as total_payment_value from order_payments group by payment_type"),
        (19, "Review Score Distribution", "select review_score, count(1) as number_of_reviews from order_reviews group by review_score"),
        (21, "Review Score Trend With Time", "select ROUND(AVG(review_score), 2) as avg_review_score, strftime('%Y-%m', review_creation_date) as review_month from order_reviews group by review_month"),
        (23, "Order Reviews", "select * from order_reviews"),
    ];

    let mut sql_id_map: HashMap<u8, Uuid> = HashMap::new();
    let mut chart_id_map: HashMap<u8, Uuid> = HashMap::new();

    for (idx, (display_order, name, content)) in sql_cells.into_iter().enumerate() {
        match execute_create_cell(
            app_state,
            CreateCellDto {
                notebook_id,
                connection_id: Some(connection_id),
                name: Some(name.to_string()),
                content: Some(content.to_string()),
                display_order: Some(display_order),
                cell_type: crate::cell::constants::CellType::Sql,
            },
        )
        .await
        {
            Ok(cell) => {
                sql_id_map.insert(idx as u8, cell.id);
            }
            Err(e) => {
                error!("Failed to create SQL cell when seeding demo data: {}", e);
                return;
            }
        }
    }

    let chart_cells_data: Vec<(u8, i32, &'static str, &'static str)> = vec![
        (
            0,
            1,
            "Avg Review Score",
            r#"{"chart_type":"METRIC","cell_id":"PLACEHOLDER","decimal_precision":2,"format":"DEFAULT"}"#,
        ),
        (
            1,
            3,
            "Total GMV",
            r#"{"chart_type":"METRIC","cell_id":"PLACEHOLDER","decimal_precision":2,"format":"CURRENCY"}"#,
        ),
        (
            2,
            5,
            "On-Time Delivery Rate",
            r#"{"chart_type":"METRIC","cell_id":"PLACEHOLDER","decimal_precision":2,"format":"PERCENTAGE"}"#,
        ),
        (
            3,
            8,
            "Monthly Revenue",
            r#"{"x_axis":"order_month","y_axis":"revenue","chart_type":"LINE","cell_id":"PLACEHOLDER","orientation":"VERTICAL","y_axis_aggregate_function":"MAX","y_axis_sort_order":"NONE","x_axis_tick_show":"SHOW","y_axis_tick_show":"SHOW","axis_minor_tick_show":"SHOW","show_dots":"SHOW"}"#,
        ),
        (
            4,
            10,
            "Orders by Status",
            r#"{"chart_type":"PIE","cell_id":"PLACEHOLDER","category":"order_status","value":"order_count","aggregate_function":"MAX"}"#,
        ),
        (
            5,
            12,
            "Avg Order Value",
            r#"{"x_axis":"order_month","y_axis":"avg_order_value","chart_type":"LINE","cell_id":"PLACEHOLDER","orientation":"VERTICAL","y_axis_aggregate_function":"MAX","y_axis_sort_order":"NONE","x_axis_tick_show":"SHOW","y_axis_tick_show":"SHOW","axis_minor_tick_show":"SHOW","show_dots":"SHOW"}"#,
        ),
        (
            6,
            14,
            "Top Products",
            r#"{"x_axis":"product_id","y_axis":"total_revenue","chart_type":"BAR","cell_id":"PLACEHOLDER","orientation":"VERTICAL","y_axis_aggregate_function":"MAX","y_axis_sort_order":"DESC","x_axis_tick_show":"HIDE","y_axis_tick_show":"SHOW","axis_minor_tick_show":"SHOW"}"#,
        ),
        (
            7,
            16,
            "Top Sellers GMV",
            r#"{"x_axis":"seller_id","y_axis":"gmv","chart_type":"BAR","cell_id":"PLACEHOLDER","orientation":"VERTICAL","y_axis_aggregate_function":"MAX","y_axis_sort_order":"DESC","x_axis_tick_show":"HIDE","y_axis_tick_show":"SHOW","axis_minor_tick_show":"SHOW"}"#,
        ),
        (
            8,
            18,
            "Revenue by Payment Type",
            r#"{"chart_type":"PIE","cell_id":"PLACEHOLDER","category":"payment_type","value":"total_payment_value","aggregate_function":"MAX"}"#,
        ),
        (
            9,
            20,
            "Review Score Distribution",
            r#"{"x_axis":"review_score","y_axis":"number_of_reviews","chart_type":"BAR","cell_id":"PLACEHOLDER","orientation":"VERTICAL","y_axis_aggregate_function":"MAX","y_axis_sort_order":"DESC","x_axis_tick_show":"SHOW","y_axis_tick_show":"SHOW","axis_minor_tick_show":"SHOW"}"#,
        ),
        (
            10,
            22,
            "Review Score Trend With Time",
            r#"{"x_axis":"review_month","y_axis":"avg_review_score","chart_type":"LINE","cell_id":"PLACEHOLDER","orientation":"VERTICAL","y_axis_aggregate_function":"MAX","y_axis_sort_order":"NONE","x_axis_tick_show":"SHOW","y_axis_tick_show":"SHOW","axis_minor_tick_show":"SHOW","show_dots":"SHOW"}"#,
        ),
    ];

    for (chart_idx, (sql_idx, display_order, name, content_template)) in
        chart_cells_data.into_iter().enumerate()
    {
        let source_cell_id = sql_id_map
            .get(&sql_idx)
            .copied()
            .unwrap_or_else(Uuid::new_v4);
        let content =
            content_template.replace("\"PLACEHOLDER\"", &format!("\"{}\"", source_cell_id));
        match execute_create_cell(
            app_state,
            CreateCellDto {
                notebook_id,
                connection_id: None,
                name: Some(name.to_string()),
                content: Some(content),
                display_order: Some(display_order),
                cell_type: crate::cell::constants::CellType::Chart,
            },
        )
        .await
        {
            Ok(cell) => {
                chart_id_map.insert(chart_idx as u8, cell.id);
            }
            Err(e) => {
                error!("Failed to create chart cell when seeding demo data: {}", e);
                return;
            }
        }
    }

    let canvas_id = canvas_result.canvas.id;
    let dashboard = match get_dashboard_by_canvas_id_entity(app_state, canvas_id).await {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to get dashboard when seeding demo data: {}", e);
            return;
        }
    };
    let mut payload = DashboardDto::from(dashboard.clone());
    payload.layout = Some(vec![
        Layout::new(chart_id_map[&0], notebook_id, 0, 0, 4, 3),
        Layout::new(chart_id_map[&1], notebook_id, 4, 0, 4, 3),
        Layout::new(chart_id_map[&2], notebook_id, 8, 0, 4, 3),
        Layout::new(chart_id_map[&3], notebook_id, 0, 3, 8, 3),
        Layout::new(chart_id_map[&4], notebook_id, 8, 3, 4, 3),
        Layout::new(chart_id_map[&5], notebook_id, 0, 6, 8, 3),
        Layout::new(chart_id_map[&8], notebook_id, 8, 6, 4, 3),
        Layout::new(chart_id_map[&6], notebook_id, 0, 9, 6, 3),
        Layout::new(chart_id_map[&7], notebook_id, 6, 9, 6, 3),
        Layout::new(chart_id_map[&9], notebook_id, 0, 12, 6, 3),
        Layout::new(chart_id_map[&10], notebook_id, 6, 12, 6, 3),
    ]);
    if let Err(e) = execute_update_dashboard(app_state, dashboard.id, payload).await {
        error!(
            "Failed to update dashboard layout when seeding demo data: {}",
            e
        );
        return;
    }

    crate::configuration::service::mark_demo_data_setup_done(app_state).await;

    info!(
        "Demo data seeded: canvas {}, notebook {}, {} cells",
        canvas_result.canvas.id, notebook_id, 23
    );
}
