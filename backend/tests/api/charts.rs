use crate::support::TestApp;
use crate::support::canvas_builder::{
    axis_chart_content, column_names, create_canvas, create_chart_cell, create_sql_cell,
    pie_chart_content, string_field,
};
use crate::support::constants::TOP_CATEGORIES_QUERY;
use reqwest::StatusCode;
use serde_json::{Value, json};

struct ChartFixture {
    app: TestApp,
    notebook_id: String,
    connection_id: String,
    sql_cell_id: String,
}

async fn chart_fixture() -> ChartFixture {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    let connection = app.create_sqlite_demo_connection().await;
    let connection_id = string_field(&connection, "id");
    let canvas = create_canvas(&app, "Charts").await;
    let sql_cell = create_sql_cell(
        &app,
        &canvas.notebook_id,
        &connection_id,
        "Top categories",
        TOP_CATEGORIES_QUERY,
    )
    .await;
    ChartFixture {
        app,
        notebook_id: canvas.notebook_id,
        connection_id,
        sql_cell_id: string_field(&sql_cell, "id"),
    }
}

async fn create_and_execute_chart(fixture: &ChartFixture, name: &str, content: Value) -> Value {
    let chart_cell = create_chart_cell(
        &fixture.app,
        &fixture.notebook_id,
        &fixture.connection_id,
        name,
        content.clone(),
    )
    .await;
    let chart_path = format!("/cell/{}", string_field(&chart_cell, "id"));

    let persisted = fixture.app.get(&chart_path).await;
    assert_eq!(persisted.status, StatusCode::OK);
    assert_eq!(persisted.body["cell_type"], "CHART");
    let persisted_content: Value = serde_json::from_str(
        persisted.body["content"]
            .as_str()
            .expect("chart content is a string"),
    )
    .expect("chart content is JSON");
    assert_eq!(persisted_content, content);

    let executed = fixture
        .app
        .post(&format!("{chart_path}/execute"), json!({}))
        .await;
    assert_eq!(executed.status, StatusCode::OK, "{}", executed.body);
    executed.body
}

#[tokio::test]
async fn bar_chart_with_two_series_persists_and_executes() {
    let fixture = chart_fixture().await;

    let result = create_and_execute_chart(
        &fixture,
        "Bar chart",
        axis_chart_content("BAR", &fixture.sql_cell_id),
    )
    .await;

    assert_eq!(
        column_names(&result),
        ["category", "product_count_sum", "product_count_max"]
    );
    assert_eq!(result["data"].as_array().expect("rows").len(), 5);
}

#[tokio::test]
async fn line_chart_with_two_series_persists_and_executes() {
    let fixture = chart_fixture().await;

    let result = create_and_execute_chart(
        &fixture,
        "Line chart",
        axis_chart_content("LINE", &fixture.sql_cell_id),
    )
    .await;

    assert_eq!(
        column_names(&result),
        ["category", "product_count_sum", "product_count_max"]
    );
}

#[tokio::test]
async fn pie_chart_persists_and_executes() {
    let fixture = chart_fixture().await;

    let result = create_and_execute_chart(
        &fixture,
        "Pie chart",
        pie_chart_content(&fixture.sql_cell_id),
    )
    .await;

    assert_eq!(column_names(&result), ["category", "product_count"]);
    assert_eq!(result["data"].as_array().expect("rows").len(), 5);
}

#[tokio::test]
async fn axis_chart_without_a_sort_order_is_rejected() {
    let fixture = chart_fixture().await;
    let mut content = axis_chart_content("BAR", &fixture.sql_cell_id);
    content
        .as_object_mut()
        .expect("chart content object")
        .remove("y_axis_sort_order");
    let chart_cell = create_chart_cell(
        &fixture.app,
        &fixture.notebook_id,
        &fixture.connection_id,
        "Unsorted bar chart",
        content,
    )
    .await;

    let executed = fixture
        .app
        .post(
            &format!("/cell/{}/execute", string_field(&chart_cell, "id")),
            json!({}),
        )
        .await;

    assert_eq!(executed.status, StatusCode::PRECONDITION_FAILED);
    assert_eq!(
        executed.body["message"],
        "ChartCell has no y-axis sort order specified"
    );
}
