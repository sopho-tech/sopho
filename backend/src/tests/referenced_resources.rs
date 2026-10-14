use crate::authorization::constants::{AccessLevel, ResourceType, SystemRole, Visibility};
use crate::authorization::dto::TenantContext;
use crate::cell::constants::{CellType, CELL_NOTEBOOK_IS_FIXED_MESSAGE};
use crate::cell::dto::{
    CellDto, ChartContent, CreateCellDto, ExecuteCellPreviewDto, MetricChartContent,
};
use crate::cell::error::CellError;
use crate::common::errors::{ExecuteChartError, GetDatabaseConnectionError};
use crate::common::AppState;
use crate::conversational_analytics::conversation::dto::CreateConversationDto;
use crate::conversational_analytics::conversation::error::ConversationError;
use crate::conversational_analytics::segment::MessageSegment;
use crate::dashboard::constants::DashboardStatus;
use crate::dashboard::error::DashboardError;
use crate::dashboard::{DashboardDto, Layout};
use crate::grant::dto::CreateGrantDto;
use crate::tests::common::app_state::{absolute_test_database_path, init_test_app_state};
use crate::tests::common::resources::{
    insert_canvas, insert_cell, insert_connection, insert_dashboard, insert_notebook,
    insert_sqlite_connection,
};
use crate::tests::common::tenancy::{context_with_role, create_tenant_with_owner, resolve_context};
use uuid::Uuid;

const ANSWER_QUERY: &str = "SELECT 1 AS answer";

fn sql_cell(notebook_id: Uuid, connection_id: Option<Uuid>) -> CreateCellDto {
    CreateCellDto {
        notebook_id,
        connection_id,
        name: Some("query".to_string()),
        content: Some(ANSWER_QUERY.to_string()),
        display_order: None,
        cell_type: CellType::Sql,
    }
}

fn metric_chart_content(source_cell_id: Uuid) -> String {
    serde_json::to_string(&ChartContent::Metric(MetricChartContent {
        cell_id: source_cell_id,
        decimal_precision: None,
        suffix: None,
        format: None,
    }))
    .unwrap()
}

fn chart_cell(notebook_id: Uuid, source_cell_id: Uuid) -> CreateCellDto {
    CreateCellDto {
        notebook_id,
        connection_id: None,
        name: Some("chart".to_string()),
        content: Some(metric_chart_content(source_cell_id)),
        display_order: None,
        cell_type: CellType::Chart,
    }
}

async fn create_cell(
    app_state: &AppState,
    context: &TenantContext,
    payload: CreateCellDto,
) -> Result<CellDto, CellError> {
    crate::cell::service::create_cell(app_state.clone(), context, payload).await
}

async fn grant_canvas(
    app_state: &AppState,
    owner: &TenantContext,
    canvas_id: Uuid,
    grantee: &TenantContext,
    access_level: AccessLevel,
) {
    crate::grant::service::create_grant(
        app_state.clone(),
        owner,
        CreateGrantDto {
            resource_type: ResourceType::Canvas.as_str().to_string(),
            resource_id: canvas_id,
            principal_user_id: grantee.user_id().get_uuid(),
            access_level: access_level.as_str().to_string(),
        },
    )
    .await
    .unwrap();
}

fn is_notebook_not_found(result: &Result<CellDto, CellError>) -> bool {
    matches!(result, Err(CellError::NotebookNotFound))
}

fn is_connection_not_found<T>(result: &Result<T, CellError>) -> bool {
    matches!(
        result,
        Err(CellError::GetConnection(
            GetDatabaseConnectionError::ConnectionNotFound
        ))
    )
}

#[tokio::test]
async fn adding_a_cell_needs_write_access_to_its_canvas() {
    let app_state = init_test_app_state("cell_create_needs_write").await;
    let fixture = create_tenant_with_owner(&app_state, "cell_create").await;
    let owner = resolve_context(&app_state, fixture.tenant_id, fixture.user_id).await;
    let canvas_id = insert_canvas(
        &app_state,
        fixture.tenant_id,
        "shared canvas",
        fixture.user_id,
        Visibility::Tenant,
    )
    .await;
    let notebook_id = insert_notebook(&app_state, fixture.tenant_id, canvas_id, "notebook").await;

    let bystander = context_with_role(
        &app_state,
        fixture.tenant_id,
        SystemRole::Member,
        "bystander",
    )
    .await;
    let viewer =
        context_with_role(&app_state, fixture.tenant_id, SystemRole::Member, "viewer").await;
    let editor =
        context_with_role(&app_state, fixture.tenant_id, SystemRole::Member, "editor").await;
    grant_canvas(&app_state, &owner, canvas_id, &viewer, AccessLevel::Viewer).await;
    grant_canvas(&app_state, &owner, canvas_id, &editor, AccessLevel::Editor).await;

    assert!(
        is_notebook_not_found(
            &create_cell(&app_state, &bystander, sql_cell(notebook_id, None)).await
        ),
        "a workspace-visible canvas is readable, not writable"
    );
    assert!(
        is_notebook_not_found(&create_cell(&app_state, &viewer, sql_cell(notebook_id, None)).await),
        "a viewer grant does not allow adding cells"
    );
    assert!(
        create_cell(&app_state, &editor, sql_cell(notebook_id, None))
            .await
            .is_ok()
    );
    assert!(create_cell(&app_state, &owner, sql_cell(notebook_id, None))
        .await
        .is_ok());
}

#[tokio::test]
async fn a_cell_only_binds_to_connections_its_author_can_read() {
    let app_state = init_test_app_state("cell_connection_binding").await;
    let fixture = create_tenant_with_owner(&app_state, "cell_connection").await;
    let private_connection_id = insert_connection(
        &app_state,
        fixture.tenant_id,
        "private",
        fixture.user_id,
        Visibility::Private,
    )
    .await;
    let shared_connection_id = insert_connection(
        &app_state,
        fixture.tenant_id,
        "shared",
        fixture.user_id,
        Visibility::Tenant,
    )
    .await;

    let member =
        context_with_role(&app_state, fixture.tenant_id, SystemRole::Member, "member").await;
    let canvas_id = insert_canvas(
        &app_state,
        fixture.tenant_id,
        "member canvas",
        member.user_id().get_uuid(),
        Visibility::Private,
    )
    .await;
    let notebook_id = insert_notebook(&app_state, fixture.tenant_id, canvas_id, "notebook").await;

    assert!(is_connection_not_found(
        &create_cell(
            &app_state,
            &member,
            sql_cell(notebook_id, Some(private_connection_id))
        )
        .await
    ));
    let cell = create_cell(
        &app_state,
        &member,
        sql_cell(notebook_id, Some(shared_connection_id)),
    )
    .await
    .unwrap();

    let mut rebound = crate::cell::service::get_cell(app_state.clone(), &member, cell.id)
        .await
        .unwrap();
    rebound.connection_id = Some(private_connection_id);
    assert!(is_connection_not_found(
        &crate::cell::service::update_cell(app_state.clone(), &member, cell.id, rebound).await
    ));

    let mut renamed = crate::cell::service::get_cell(app_state.clone(), &member, cell.id)
        .await
        .unwrap();
    renamed.name = Some("renamed".to_string());
    let updated = crate::cell::service::update_cell(app_state.clone(), &member, cell.id, renamed)
        .await
        .unwrap();
    assert_eq!(updated.connection_id, Some(shared_connection_id));
    assert_eq!(updated.name.as_deref(), Some("renamed"));
}

#[tokio::test]
async fn a_cell_cannot_move_to_another_notebook() {
    let app_state = init_test_app_state("cell_notebook_is_fixed").await;
    let fixture = create_tenant_with_owner(&app_state, "cell_move").await;
    let owner = resolve_context(&app_state, fixture.tenant_id, fixture.user_id).await;
    let first_canvas_id = insert_canvas(
        &app_state,
        fixture.tenant_id,
        "first",
        fixture.user_id,
        Visibility::Private,
    )
    .await;
    let second_canvas_id = insert_canvas(
        &app_state,
        fixture.tenant_id,
        "second",
        fixture.user_id,
        Visibility::Private,
    )
    .await;
    let first_notebook_id =
        insert_notebook(&app_state, fixture.tenant_id, first_canvas_id, "first").await;
    let second_notebook_id =
        insert_notebook(&app_state, fixture.tenant_id, second_canvas_id, "second").await;
    let cell_id = insert_cell(&app_state, fixture.tenant_id, first_notebook_id, "cell").await;

    let mut moved = crate::cell::service::get_cell(app_state.clone(), &owner, cell_id)
        .await
        .unwrap();
    moved.notebook_id = second_notebook_id;
    let result = crate::cell::service::update_cell(app_state.clone(), &owner, cell_id, moved).await;
    assert!(
        matches!(&result, Err(CellError::InvalidRequest(message)) if message == CELL_NOTEBOOK_IS_FIXED_MESSAGE)
    );

    let unchanged = crate::cell::service::get_cell(app_state.clone(), &owner, cell_id)
        .await
        .unwrap();
    assert_eq!(unchanged.notebook_id, first_notebook_id);
}

#[tokio::test]
async fn a_chart_only_reads_from_cells_of_its_own_notebook() {
    let test_name = "chart_source_scope";
    let app_state = init_test_app_state(test_name).await;
    let fixture = create_tenant_with_owner(&app_state, "chart_source").await;
    let owner = resolve_context(&app_state, fixture.tenant_id, fixture.user_id).await;
    let connection_id = insert_sqlite_connection(
        &app_state,
        fixture.tenant_id,
        "answers",
        fixture.user_id,
        Visibility::Tenant,
        &absolute_test_database_path(test_name),
    )
    .await;

    let private_canvas_id = insert_canvas(
        &app_state,
        fixture.tenant_id,
        "owner only",
        fixture.user_id,
        Visibility::Private,
    )
    .await;
    let private_notebook_id =
        insert_notebook(&app_state, fixture.tenant_id, private_canvas_id, "private").await;
    let private_source = create_cell(
        &app_state,
        &owner,
        sql_cell(private_notebook_id, Some(connection_id)),
    )
    .await
    .unwrap();

    let member =
        context_with_role(&app_state, fixture.tenant_id, SystemRole::Member, "member").await;
    let member_canvas_id = insert_canvas(
        &app_state,
        fixture.tenant_id,
        "member canvas",
        member.user_id().get_uuid(),
        Visibility::Private,
    )
    .await;
    let member_notebook_id =
        insert_notebook(&app_state, fixture.tenant_id, member_canvas_id, "member").await;
    let own_source = create_cell(
        &app_state,
        &member,
        sql_cell(member_notebook_id, Some(connection_id)),
    )
    .await
    .unwrap();

    let own_chart = create_cell(
        &app_state,
        &member,
        chart_cell(member_notebook_id, own_source.id),
    )
    .await
    .unwrap();
    assert!(
        crate::cell::service::execute_cell(app_state.clone(), &member, own_chart.id)
            .await
            .is_ok(),
        "a chart over a cell of its own notebook executes"
    );

    let borrowing_chart = create_cell(
        &app_state,
        &member,
        chart_cell(member_notebook_id, private_source.id),
    )
    .await
    .unwrap();
    assert!(matches!(
        crate::cell::service::execute_cell(app_state.clone(), &member, borrowing_chart.id).await,
        Err(CellError::ExecuteChart(ExecuteChartError::CellNotFound))
    ));

    let preview = crate::cell::service::execute_cell_preview(
        app_state.clone(),
        &member,
        own_chart.id,
        ExecuteCellPreviewDto {
            content: metric_chart_content(private_source.id),
            cell_type: CellType::Chart,
        },
    )
    .await;
    assert!(matches!(
        preview,
        Err(CellError::ExecuteChart(ExecuteChartError::CellNotFound))
    ));
}

#[tokio::test]
async fn a_conversation_only_starts_on_a_connection_its_author_can_read() {
    let app_state = init_test_app_state("conversation_connection_binding").await;
    let fixture = create_tenant_with_owner(&app_state, "conversation_connection").await;
    let private_connection_id = insert_connection(
        &app_state,
        fixture.tenant_id,
        "private",
        fixture.user_id,
        Visibility::Private,
    )
    .await;
    let shared_connection_id = insert_connection(
        &app_state,
        fixture.tenant_id,
        "shared",
        fixture.user_id,
        Visibility::Tenant,
    )
    .await;
    let member =
        context_with_role(&app_state, fixture.tenant_id, SystemRole::Member, "member").await;

    let start = |connection_id: Uuid| {
        crate::conversational_analytics::conversation::service::create_conversation(
            app_state.clone(),
            &member,
            CreateConversationDto {
                connection_id,
                segments: vec![MessageSegment::Text {
                    text: "How many orders shipped?".to_string(),
                }],
            },
        )
    };

    assert!(matches!(
        start(private_connection_id).await,
        Err(ConversationError::InvalidRequest(_))
    ));
    assert!(
        matches!(
            start(shared_connection_id).await,
            Err(ConversationError::AiNotLive)
        ),
        "a readable connection passes the connection check and stops at the missing AI setup"
    );
}

#[tokio::test]
async fn a_dashboard_only_lays_out_charts_of_its_own_canvas() {
    let app_state = init_test_app_state("dashboard_layout_scope").await;
    let fixture = create_tenant_with_owner(&app_state, "dashboard_layout").await;
    let owner = resolve_context(&app_state, fixture.tenant_id, fixture.user_id).await;
    let canvas_id = insert_canvas(
        &app_state,
        fixture.tenant_id,
        "canvas",
        fixture.user_id,
        Visibility::Private,
    )
    .await;
    let other_canvas_id = insert_canvas(
        &app_state,
        fixture.tenant_id,
        "other",
        fixture.user_id,
        Visibility::Private,
    )
    .await;
    let notebook_id = insert_notebook(&app_state, fixture.tenant_id, canvas_id, "notebook").await;
    let other_notebook_id =
        insert_notebook(&app_state, fixture.tenant_id, other_canvas_id, "other").await;
    let dashboard_id =
        insert_dashboard(&app_state, fixture.tenant_id, canvas_id, "dashboard").await;
    let chart_id = insert_cell(&app_state, fixture.tenant_id, notebook_id, "chart").await;
    let other_chart_id = insert_cell(
        &app_state,
        fixture.tenant_id,
        other_notebook_id,
        "other chart",
    )
    .await;

    let save_layout = |cell_id: Uuid, layout_notebook_id: Uuid| {
        crate::dashboard::service::update_dashboard(
            app_state.clone(),
            &owner,
            dashboard_id,
            DashboardDto {
                id: dashboard_id,
                name: Some("dashboard".to_string()),
                description: Some(String::new()),
                layout: Some(vec![Layout::new(cell_id, layout_notebook_id, 0, 0, 4, 3)]),
                status: DashboardStatus::Active,
            },
        )
    };

    assert!(save_layout(chart_id, notebook_id).await.is_ok());
    assert!(matches!(
        save_layout(other_chart_id, other_notebook_id).await,
        Err(DashboardError::LayoutOutsideCanvas)
    ));
    assert!(matches!(
        save_layout(chart_id, other_notebook_id).await,
        Err(DashboardError::LayoutOutsideCanvas)
    ));
}
