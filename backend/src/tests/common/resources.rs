use crate::authorization::constants::Visibility;
use crate::common::time_utils;
use crate::common::AppState;
use crate::entity;
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

const ACTIVE_STATUS: &str = "ACTIVE";
const UNUSED_SQLITE_DATABASE: &str = "./data/unused.db";

pub async fn insert_canvas(
    app_state: &AppState,
    tenant_id: Uuid,
    name: &str,
    owner_user_id: Uuid,
    visibility: Visibility,
) -> Uuid {
    let canvas_id = Uuid::new_v4();
    entity::canvas::ActiveModel {
        id: Set(canvas_id),
        tenant_id: Set(tenant_id),
        owner_user_id: Set(owner_user_id),
        visibility: Set(visibility.as_str().to_string()),
        name: Set(name.to_string()),
        description: Set(None),
        status: Set(ACTIVE_STATUS.to_string()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();
    canvas_id
}

pub async fn insert_notebook(
    app_state: &AppState,
    tenant_id: Uuid,
    canvas_id: Uuid,
    name: &str,
) -> Uuid {
    let notebook_id = Uuid::new_v4();
    entity::notebook::ActiveModel {
        id: Set(notebook_id),
        tenant_id: Set(tenant_id),
        canvas_id: Set(canvas_id),
        name: Set(name.to_string()),
        description: Set(None),
        status: Set(ACTIVE_STATUS.to_string()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();
    notebook_id
}

pub async fn insert_dashboard(
    app_state: &AppState,
    tenant_id: Uuid,
    canvas_id: Uuid,
    name: &str,
) -> Uuid {
    let dashboard_id = Uuid::new_v4();
    entity::dashboard::ActiveModel {
        id: Set(dashboard_id),
        tenant_id: Set(tenant_id),
        canvas_id: Set(canvas_id),
        name: Set(name.to_string()),
        description: Set(String::new()),
        layout: Set(None),
        status: Set(ACTIVE_STATUS.to_string()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();
    dashboard_id
}

pub async fn insert_cell(
    app_state: &AppState,
    tenant_id: Uuid,
    notebook_id: Uuid,
    name: &str,
) -> Uuid {
    let cell_id = Uuid::new_v4();
    entity::cell::ActiveModel {
        id: Set(cell_id),
        tenant_id: Set(tenant_id),
        notebook_id: Set(notebook_id),
        connection_id: Set(None),
        display_order: Set(0),
        name: Set(Some(name.to_string())),
        content: Set(None),
        cell_type: Set(crate::cell::constants::CellType::Sql.to_string()),
        status: Set(ACTIVE_STATUS.to_string()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();
    cell_id
}

pub async fn insert_connection(
    app_state: &AppState,
    tenant_id: Uuid,
    name: &str,
    owner_user_id: Uuid,
    visibility: Visibility,
) -> Uuid {
    insert_sqlite_connection(
        app_state,
        tenant_id,
        name,
        owner_user_id,
        visibility,
        UNUSED_SQLITE_DATABASE,
    )
    .await
}

pub async fn insert_sqlite_connection(
    app_state: &AppState,
    tenant_id: Uuid,
    name: &str,
    owner_user_id: Uuid,
    visibility: Visibility,
    database_path: &str,
) -> Uuid {
    let connection_id = Uuid::new_v4();
    entity::connection::ActiveModel {
        id: Set(connection_id),
        tenant_id: Set(tenant_id),
        owner_user_id: Set(owner_user_id),
        visibility: Set(visibility.as_str().to_string()),
        name: Set(name.to_string()),
        description: Set(None),
        username: Set(None),
        password: Set(None),
        host: Set(None),
        port: Set(None),
        database: Set(database_path.to_string()),
        schema: Set(None),
        source_type: Set("SQLITE".to_string()),
        status: Set(ACTIVE_STATUS.to_string()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();
    connection_id
}

pub async fn insert_conversation(
    app_state: &AppState,
    tenant_id: Uuid,
    name: &str,
    owner_user_id: Uuid,
    visibility: Visibility,
    connection_id: Uuid,
) -> Uuid {
    let conversation_id = Uuid::new_v4();
    entity::conversation::ActiveModel {
        id: Set(conversation_id),
        tenant_id: Set(tenant_id),
        owner_user_id: Set(owner_user_id),
        visibility: Set(visibility.as_str().to_string()),
        connection_id: Set(connection_id),
        name: Set(name.to_string()),
        status: Set(ACTIVE_STATUS.to_string()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();
    conversation_id
}
