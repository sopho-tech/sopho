use crate::support::TestApp;
use reqwest::StatusCode;
use serde_json::{Value, json};

pub struct CreatedCanvas {
    pub canvas_id: String,
    pub notebook_id: String,
    pub dashboard: Value,
}

pub async fn create_canvas(app: &TestApp, name: &str) -> CreatedCanvas {
    let created = app
        .post(
            "/canvas",
            json!({ "name": name, "description": "Created by the API tests" }),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    let canvas_id = string_field(&created.body, "id");
    let notebooks = app.get(&format!("/notebook/canvas/{canvas_id}")).await;
    assert_eq!(notebooks.status, StatusCode::OK, "{}", notebooks.body);
    let notebook_id = string_field(&notebooks.body[0], "id");
    let dashboard = app.get(&format!("/dashboard/canvas/{canvas_id}")).await;
    assert_eq!(dashboard.status, StatusCode::OK, "{}", dashboard.body);
    CreatedCanvas {
        canvas_id,
        notebook_id,
        dashboard: dashboard.body,
    }
}

pub async fn create_sql_cell(
    app: &TestApp,
    notebook_id: &str,
    connection_id: &str,
    name: &str,
    query: &str,
) -> Value {
    create_cell(
        app,
        json!({
            "notebook_id": notebook_id,
            "connection_id": connection_id,
            "name": name,
            "content": query,
            "cell_type": "SQL",
        }),
    )
    .await
}

pub async fn create_chart_cell(
    app: &TestApp,
    notebook_id: &str,
    connection_id: &str,
    name: &str,
    chart_content: Value,
) -> Value {
    create_cell(
        app,
        json!({
            "notebook_id": notebook_id,
            "connection_id": connection_id,
            "name": name,
            "content": chart_content.to_string(),
            "cell_type": "CHART",
        }),
    )
    .await
}

async fn create_cell(app: &TestApp, payload: Value) -> Value {
    let created = app.post("/cell", payload).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    created.body
}

pub async fn place_chart_on_dashboard(app: &TestApp, canvas: &CreatedCanvas, chart_cell_id: &str) {
    let mut dashboard = canvas.dashboard.clone();
    dashboard["layout"] = json!([
        {
            "cell_id": chart_cell_id,
            "notebook_id": canvas.notebook_id,
            "x_position": 0,
            "y_position": 0,
            "x_size": 6,
            "y_size": 4,
        },
    ]);
    let updated = app
        .put(
            &format!("/dashboard/{}", string_field(&canvas.dashboard, "id")),
            dashboard,
        )
        .await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.body);
}

pub fn axis_chart_content(chart_type: &str, sql_cell_id: &str) -> Value {
    json!({
        "chart_type": chart_type,
        "cell_id": sql_cell_id,
        "x_axis": "category",
        "x_axis_alias": "category",
        "series": [
            {
                "column": "product_count",
                "aggregate_function": "SUM",
                "label": "Products",
                "alias": "product_count_sum",
                "color_index": null,
            },
            {
                "column": "product_count",
                "aggregate_function": "MAX",
                "label": "Largest group",
                "alias": "product_count_max",
                "color_index": null,
            },
        ],
        "x_axis_title": null,
        "y_axis_title": null,
        "y_axis_sort_by": "product_count_sum",
        "y_axis_sort_order": "DESC",
    })
}

pub fn order_status_chart_content(sql_cell_id: &str) -> Value {
    json!({
        "chart_type": "BAR",
        "cell_id": sql_cell_id,
        "x_axis": "order_status",
        "x_axis_alias": "order_status",
        "series": [
            {
                "column": "order_count",
                "aggregate_function": "SUM",
                "label": "Orders",
                "alias": "order_count_sum",
                "color_index": null,
            },
        ],
        "x_axis_title": null,
        "y_axis_title": null,
        "y_axis_sort_by": "order_count_sum",
        "y_axis_sort_order": "DESC",
    })
}

pub fn pie_chart_content(sql_cell_id: &str) -> Value {
    json!({
        "chart_type": "PIE",
        "cell_id": sql_cell_id,
        "category": "category",
        "value": "product_count",
        "aggregate_function": "SUM",
    })
}

pub fn string_field(value: &Value, field: &str) -> String {
    value[field]
        .as_str()
        .unwrap_or_else(|| panic!("{field} is missing from {value}"))
        .to_string()
}

pub fn column_names(execution_result: &Value) -> Vec<String> {
    execution_result["columns"]
        .as_array()
        .expect("result columns")
        .iter()
        .map(|column| string_field(column, "column_name"))
        .collect()
}
