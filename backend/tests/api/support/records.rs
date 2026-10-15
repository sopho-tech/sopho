use crate::support::constants::{ACTIVE_STATUS, USER_PRINCIPAL};
use crate::support::database::{insert_row, now, update_rows};
use crate::support::test_app::TestApp;
use crate::support::workspaces::with_system_attribution;
use sea_orm::Value;
use uuid::Uuid;

pub async fn insert_canvas(
    app: &TestApp,
    tenant_id: Uuid,
    name: &str,
    owner_user_id: Uuid,
    visibility: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    insert_row(
        app,
        "canvas",
        &with_system_attribution(vec![
            ("id", id.into()),
            ("tenant_id", tenant_id.into()),
            ("owner_user_id", owner_user_id.into()),
            ("visibility", visibility.into()),
            ("name", name.into()),
            ("description", Value::String(None)),
            ("status", ACTIVE_STATUS.into()),
        ]),
    )
    .await;
    id
}

pub async fn insert_notebook(app: &TestApp, tenant_id: Uuid, canvas_id: Uuid, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    insert_row(
        app,
        "notebook",
        &with_system_attribution(vec![
            ("id", id.into()),
            ("tenant_id", tenant_id.into()),
            ("canvas_id", canvas_id.into()),
            ("name", name.into()),
            ("description", Value::String(None)),
            ("status", ACTIVE_STATUS.into()),
        ]),
    )
    .await;
    id
}

pub async fn insert_dashboard(app: &TestApp, tenant_id: Uuid, canvas_id: Uuid, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    insert_row(
        app,
        "dashboard",
        &with_system_attribution(vec![
            ("id", id.into()),
            ("tenant_id", tenant_id.into()),
            ("canvas_id", canvas_id.into()),
            ("name", name.into()),
            ("description", "".into()),
            ("layout", Value::Json(None)),
            ("status", ACTIVE_STATUS.into()),
        ]),
    )
    .await;
    id
}

pub async fn insert_sql_cell(
    app: &TestApp,
    tenant_id: Uuid,
    notebook_id: Uuid,
    name: &str,
    content: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    insert_row(
        app,
        "cell",
        &with_system_attribution(vec![
            ("id", id.into()),
            ("tenant_id", tenant_id.into()),
            ("notebook_id", notebook_id.into()),
            ("connection_id", Value::Uuid(None)),
            ("display_order", 0.into()),
            ("name", name.into()),
            ("content", content.into()),
            ("cell_type", "SQL".into()),
            ("status", ACTIVE_STATUS.into()),
        ]),
    )
    .await;
    id
}

pub async fn insert_conversation(
    app: &TestApp,
    tenant_id: Uuid,
    name: &str,
    owner_user_id: Uuid,
    visibility: &str,
    connection_id: Uuid,
) -> Uuid {
    let id = Uuid::new_v4();
    insert_row(
        app,
        "conversation",
        &with_system_attribution(vec![
            ("id", id.into()),
            ("tenant_id", tenant_id.into()),
            ("owner_user_id", owner_user_id.into()),
            ("visibility", visibility.into()),
            ("connection_id", connection_id.into()),
            ("name", name.into()),
            ("status", ACTIVE_STATUS.into()),
        ]),
    )
    .await;
    id
}

pub async fn insert_grant(
    app: &TestApp,
    tenant_id: Uuid,
    resource_type: &str,
    resource_id: Uuid,
    principal_user_id: Uuid,
    access_level: &str,
) {
    insert_row(
        app,
        "resource_grant",
        &[
            ("id", Uuid::new_v4().into()),
            ("tenant_id", tenant_id.into()),
            ("resource_type", resource_type.into()),
            ("resource_id", resource_id.into()),
            ("principal_type", USER_PRINCIPAL.into()),
            ("principal_id", principal_user_id.into()),
            ("access_level", access_level.into()),
            ("granted_by_user_id", Uuid::new_v4().into()),
            ("created_at", now().into()),
            ("updated_at", now().into()),
        ],
    )
    .await;
}

pub async fn insert_suggested_question(
    app: &TestApp,
    tenant_id: Uuid,
    connection_id: Uuid,
    question_text: &str,
) {
    insert_row(
        app,
        "suggested_question",
        &with_system_attribution(vec![
            ("id", Uuid::new_v4().into()),
            ("tenant_id", tenant_id.into()),
            ("connection_id", connection_id.into()),
            ("question_text", question_text.into()),
            ("generated_at", now().into()),
        ]),
    )
    .await;
}

pub async fn insert_dashboard_summary(
    app: &TestApp,
    tenant_id: Uuid,
    dashboard_id: Uuid,
    summary_text: &str,
) {
    insert_row(
        app,
        "ai_summary",
        &with_system_attribution(vec![
            ("id", Uuid::new_v4().into()),
            ("tenant_id", tenant_id.into()),
            ("entity_type", "DASHBOARD".into()),
            ("entity_id", dashboard_id.into()),
            ("status", "READY".into()),
            ("summary_text", summary_text.into()),
            ("error_message", Value::String(None)),
            ("summarized_entity_count", Value::Int(None)),
            ("user_prompt", Value::String(None)),
            ("user_prompt_used", Value::String(None)),
            ("generated_at", Value::ChronoDateTimeWithTimeZone(None)),
            ("requested_at", now().into()),
        ]),
    )
    .await;
}

pub async fn set_owner_and_visibility(
    app: &TestApp,
    table: &str,
    id: Uuid,
    owner_user_id: Uuid,
    visibility: &str,
) {
    let updated = update_rows(
        app,
        table,
        &[
            ("owner_user_id", owner_user_id.into()),
            ("visibility", visibility.into()),
        ],
        &[("id", id.into())],
    )
    .await;
    assert_eq!(updated, 1, "expected one {table} row to update");
}
