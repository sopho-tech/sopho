use crate::support::canvas_builder::string_field;
use crate::support::constants::{
    ANSWER_QUERY, CANVAS_RESOURCE, CELL_NOT_FOUND_MESSAGE, CELL_NOTEBOOK_IS_FIXED_MESSAGE,
    CONNECTION_NOT_FOUND_MESSAGE, EDITOR_ACCESS, MEMBER_ROLE, NOTEBOOK_NOT_FOUND_MESSAGE,
    OWNER_ROLE, PRIVATE_VISIBILITY, TENANT_VISIBILITY, VIEWER_ACCESS,
};
use crate::support::records::{
    insert_canvas, insert_dashboard, insert_notebook, insert_sql_cell, set_owner_and_visibility,
};
use crate::support::tenants::{create_tenant, join_tenant};
use crate::support::test_app::JsonResponse;
use crate::support::{TestApp, UserSession};
use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

fn uuid_field(value: &Value, field: &str) -> Uuid {
    string_field(value, field)
        .parse()
        .unwrap_or_else(|error| panic!("{field} is not a UUID: {error}"))
}

fn metric_chart_content(source_cell_id: Uuid) -> String {
    json!({
        "chart_type": "METRIC",
        "cell_id": source_cell_id,
        "decimal_precision": null,
        "suffix": null,
        "format": null,
    })
    .to_string()
}

async fn create_sql_cell(
    session: &UserSession,
    notebook_id: Uuid,
    connection_id: Option<Uuid>,
) -> JsonResponse {
    session
        .post(
            "/cell",
            json!({
                "notebook_id": notebook_id,
                "connection_id": connection_id,
                "name": "query",
                "content": ANSWER_QUERY,
                "cell_type": "SQL",
            }),
        )
        .await
}

async fn create_chart_cell(
    session: &UserSession,
    notebook_id: Uuid,
    source_cell_id: Uuid,
) -> JsonResponse {
    session
        .post(
            "/cell",
            json!({
                "notebook_id": notebook_id,
                "connection_id": null,
                "name": "chart",
                "content": metric_chart_content(source_cell_id),
                "cell_type": "CHART",
            }),
        )
        .await
}

fn created_id(response: &JsonResponse) -> Uuid {
    assert_eq!(response.status, StatusCode::CREATED, "{}", response.body);
    uuid_field(&response.body, "id")
}

async fn cell(session: &UserSession, cell_id: Uuid) -> Value {
    let cell = session.get(&format!("/cell/{cell_id}")).await;
    assert_eq!(cell.status, StatusCode::OK, "{}", cell.body);
    cell.body
}

async fn connection_with_visibility(
    app: &TestApp,
    creator: &UserSession,
    owner_user_id: Uuid,
    name: &str,
    visibility: &str,
) -> Uuid {
    let connection = app
        .create_named_sqlite_demo_connection_as(creator, name)
        .await;
    let connection_id = uuid_field(&connection, "id");
    set_owner_and_visibility(app, "connection", connection_id, owner_user_id, visibility).await;
    connection_id
}

async fn grant_canvas(
    owner: &UserSession,
    canvas_id: Uuid,
    grantee_user_id: Uuid,
    access_level: &str,
) {
    let granted = owner
        .post(
            "/grant",
            json!({
                "resource_type": CANVAS_RESOURCE,
                "resource_id": canvas_id,
                "principal_user_id": grantee_user_id,
                "access_level": access_level,
            }),
        )
        .await;
    assert_eq!(granted.status, StatusCode::CREATED, "{}", granted.body);
}

#[tokio::test]
async fn adding_a_cell_needs_write_access_to_its_canvas() {
    let app = TestApp::spawn().await;
    let tenant_id = create_tenant(&app, "cell_create").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let canvas_id = insert_canvas(
        &app,
        tenant_id,
        "shared canvas",
        owner.user.id,
        TENANT_VISIBILITY,
    )
    .await;
    let notebook_id = insert_notebook(&app, tenant_id, canvas_id, "notebook").await;
    let bystander = join_tenant(&app, tenant_id, "bystander", MEMBER_ROLE).await;
    let viewer = join_tenant(&app, tenant_id, "viewer", MEMBER_ROLE).await;
    let editor = join_tenant(&app, tenant_id, "editor", MEMBER_ROLE).await;
    grant_canvas(&owner.session, canvas_id, viewer.user.id, VIEWER_ACCESS).await;
    grant_canvas(&owner.session, canvas_id, editor.user.id, EDITOR_ACCESS).await;

    let by_bystander = create_sql_cell(&bystander.session, notebook_id, None).await;
    assert_eq!(
        by_bystander.status,
        StatusCode::BAD_REQUEST,
        "a tenant-visible canvas is readable, not writable"
    );
    assert_eq!(by_bystander.body["error"], NOTEBOOK_NOT_FOUND_MESSAGE);
    let by_viewer = create_sql_cell(&viewer.session, notebook_id, None).await;
    assert_eq!(
        by_viewer.status,
        StatusCode::BAD_REQUEST,
        "a viewer grant does not allow adding cells"
    );
    assert_eq!(by_viewer.body["error"], NOTEBOOK_NOT_FOUND_MESSAGE);
    created_id(&create_sql_cell(&editor.session, notebook_id, None).await);
    created_id(&create_sql_cell(&owner.session, notebook_id, None).await);
}

#[tokio::test]
async fn a_cell_only_binds_to_connections_its_author_can_read() {
    let app = TestApp::spawn().await;
    let tenant_id = create_tenant(&app, "cell_connection").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let private_connection_id = connection_with_visibility(
        &app,
        &owner.session,
        owner.user.id,
        "private",
        PRIVATE_VISIBILITY,
    )
    .await;
    let shared_connection_id = connection_with_visibility(
        &app,
        &owner.session,
        owner.user.id,
        "shared",
        TENANT_VISIBILITY,
    )
    .await;
    let member = join_tenant(&app, tenant_id, "member", MEMBER_ROLE).await;
    let canvas_id = insert_canvas(
        &app,
        tenant_id,
        "member canvas",
        member.user.id,
        PRIVATE_VISIBILITY,
    )
    .await;
    let notebook_id = insert_notebook(&app, tenant_id, canvas_id, "notebook").await;

    let on_private_connection =
        create_sql_cell(&member.session, notebook_id, Some(private_connection_id)).await;
    assert_eq!(on_private_connection.status, StatusCode::NOT_FOUND);
    assert_eq!(
        on_private_connection.body["error"],
        CONNECTION_NOT_FOUND_MESSAGE
    );
    let cell_id = created_id(
        &create_sql_cell(&member.session, notebook_id, Some(shared_connection_id)).await,
    );
    let cell_path = format!("/cell/{cell_id}");

    let mut rebound = cell(&member.session, cell_id).await;
    rebound["connection_id"] = json!(private_connection_id);
    let rebinding = member.session.put(&cell_path, rebound).await;
    assert_eq!(rebinding.status, StatusCode::NOT_FOUND);
    assert_eq!(rebinding.body["error"], CONNECTION_NOT_FOUND_MESSAGE);

    let mut renamed = cell(&member.session, cell_id).await;
    renamed["name"] = json!("renamed");
    let updated = member.session.put(&cell_path, renamed).await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.body);
    assert_eq!(
        updated.body["connection_id"],
        shared_connection_id.to_string()
    );
    assert_eq!(updated.body["name"], "renamed");
}

#[tokio::test]
async fn a_cell_cannot_move_to_another_notebook() {
    let app = TestApp::spawn().await;
    let tenant_id = create_tenant(&app, "cell_move").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let first_canvas_id =
        insert_canvas(&app, tenant_id, "first", owner.user.id, PRIVATE_VISIBILITY).await;
    let second_canvas_id =
        insert_canvas(&app, tenant_id, "second", owner.user.id, PRIVATE_VISIBILITY).await;
    let first_notebook_id = insert_notebook(&app, tenant_id, first_canvas_id, "first").await;
    let second_notebook_id = insert_notebook(&app, tenant_id, second_canvas_id, "second").await;
    let cell_id = insert_sql_cell(&app, tenant_id, first_notebook_id, "cell", ANSWER_QUERY).await;

    let mut moved = cell(&owner.session, cell_id).await;
    moved["notebook_id"] = json!(second_notebook_id);
    let moving = owner.session.put(&format!("/cell/{cell_id}"), moved).await;
    assert_eq!(moving.status, StatusCode::BAD_REQUEST);
    assert_eq!(moving.body["error"], CELL_NOTEBOOK_IS_FIXED_MESSAGE);

    assert_eq!(
        cell(&owner.session, cell_id).await["notebook_id"],
        first_notebook_id.to_string()
    );
}

#[tokio::test]
async fn a_chart_only_reads_from_cells_of_its_own_notebook() {
    let app = TestApp::spawn().await;
    let tenant_id = create_tenant(&app, "chart_source").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let connection_id = connection_with_visibility(
        &app,
        &owner.session,
        owner.user.id,
        "answers",
        TENANT_VISIBILITY,
    )
    .await;
    let private_canvas_id = insert_canvas(
        &app,
        tenant_id,
        "owner only",
        owner.user.id,
        PRIVATE_VISIBILITY,
    )
    .await;
    let private_notebook_id = insert_notebook(&app, tenant_id, private_canvas_id, "private").await;
    let private_source_id = created_id(
        &create_sql_cell(&owner.session, private_notebook_id, Some(connection_id)).await,
    );

    let member = join_tenant(&app, tenant_id, "member", MEMBER_ROLE).await;
    let member_canvas_id = insert_canvas(
        &app,
        tenant_id,
        "member canvas",
        member.user.id,
        PRIVATE_VISIBILITY,
    )
    .await;
    let member_notebook_id = insert_notebook(&app, tenant_id, member_canvas_id, "member").await;
    let own_source_id = created_id(
        &create_sql_cell(&member.session, member_notebook_id, Some(connection_id)).await,
    );
    let own_chart_id =
        created_id(&create_chart_cell(&member.session, member_notebook_id, own_source_id).await);
    let own_chart_result = member
        .session
        .post(&format!("/cell/{own_chart_id}/execute"), json!({}))
        .await;
    assert_eq!(
        own_chart_result.status,
        StatusCode::OK,
        "a chart over a cell of its own notebook executes: {}",
        own_chart_result.body
    );

    let borrowing_chart_id = created_id(
        &create_chart_cell(&member.session, member_notebook_id, private_source_id).await,
    );
    let borrowed_result = member
        .session
        .post(&format!("/cell/{borrowing_chart_id}/execute"), json!({}))
        .await;
    assert_eq!(borrowed_result.status, StatusCode::NOT_FOUND);
    assert_eq!(borrowed_result.body["error"], CELL_NOT_FOUND_MESSAGE);

    let preview = member
        .session
        .post(
            &format!("/cell/{own_chart_id}/execute-preview"),
            json!({
                "content": metric_chart_content(private_source_id),
                "cell_type": "CHART",
            }),
        )
        .await;
    assert_eq!(preview.status, StatusCode::NOT_FOUND);
    assert_eq!(preview.body["error"], CELL_NOT_FOUND_MESSAGE);
}

#[tokio::test]
async fn a_conversation_only_starts_on_a_connection_its_author_can_read() {
    let app = TestApp::spawn().await;
    let tenant_id = create_tenant(&app, "conversation_connection").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let private_connection_id = connection_with_visibility(
        &app,
        &owner.session,
        owner.user.id,
        "private",
        PRIVATE_VISIBILITY,
    )
    .await;
    let shared_connection_id = connection_with_visibility(
        &app,
        &owner.session,
        owner.user.id,
        "shared",
        TENANT_VISIBILITY,
    )
    .await;
    let member = join_tenant(&app, tenant_id, "member", MEMBER_ROLE).await;
    let start = async |connection_id: Uuid| {
        member
            .session
            .post(
                "/conversational_analytics",
                json!({
                    "connection_id": connection_id,
                    "segments": [{ "type": "TEXT", "text": "How many orders shipped?" }],
                }),
            )
            .await
    };

    assert_eq!(
        start(private_connection_id).await.status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        start(shared_connection_id).await.status,
        StatusCode::FORBIDDEN,
        "a readable connection passes the connection check and stops at the missing AI setup"
    );
}

#[tokio::test]
async fn a_dashboard_only_lays_out_charts_of_its_own_canvas() {
    let app = TestApp::spawn().await;
    let tenant_id = create_tenant(&app, "dashboard_layout").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let canvas_id =
        insert_canvas(&app, tenant_id, "canvas", owner.user.id, PRIVATE_VISIBILITY).await;
    let other_canvas_id =
        insert_canvas(&app, tenant_id, "other", owner.user.id, PRIVATE_VISIBILITY).await;
    let notebook_id = insert_notebook(&app, tenant_id, canvas_id, "notebook").await;
    let other_notebook_id = insert_notebook(&app, tenant_id, other_canvas_id, "other").await;
    let dashboard_id = insert_dashboard(&app, tenant_id, canvas_id, "dashboard").await;
    let chart_id = insert_sql_cell(&app, tenant_id, notebook_id, "chart", ANSWER_QUERY).await;
    let other_chart_id = insert_sql_cell(
        &app,
        tenant_id,
        other_notebook_id,
        "other chart",
        ANSWER_QUERY,
    )
    .await;
    let save_layout = async |cell_id: Uuid, layout_notebook_id: Uuid| {
        owner
            .session
            .put(
                &format!("/dashboard/{dashboard_id}"),
                json!({
                    "id": dashboard_id,
                    "name": "dashboard",
                    "description": "",
                    "layout": [{
                        "cell_id": cell_id,
                        "notebook_id": layout_notebook_id,
                        "x_position": 0,
                        "y_position": 0,
                        "x_size": 4,
                        "y_size": 3,
                    }],
                    "status": "ACTIVE",
                }),
            )
            .await
    };

    let saved = save_layout(chart_id, notebook_id).await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.body);
    assert_eq!(
        save_layout(other_chart_id, other_notebook_id).await.status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        save_layout(chart_id, other_notebook_id).await.status,
        StatusCode::BAD_REQUEST
    );
}
