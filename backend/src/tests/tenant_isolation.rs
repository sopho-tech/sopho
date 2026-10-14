use crate::authorization::constants::{SystemRole, Visibility};
use crate::authorization::dto::TenantContext;
use crate::authorization::errors::TenantContextError;
use crate::common::time_utils;
use crate::common::AppState;
use crate::entity;
use crate::tests::common::app_state::init_test_app_state;
use crate::tests::common::resources::{
    insert_canvas, insert_connection, insert_conversation, insert_dashboard, insert_notebook,
};
use crate::tests::common::tenancy::{create_tenant_with_owner, resolve_context, TenantFixture};
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

async fn tenant_context_for(app_state: &AppState, fixture: &TenantFixture) -> TenantContext {
    resolve_context(app_state, fixture.tenant_id, fixture.user_id).await
}

fn assert_unreachable<T: std::fmt::Debug, E: std::fmt::Debug>(
    result: Result<T, E>,
    resource: &str,
) {
    assert!(
        result.is_err(),
        "{resource} must not be readable from another tenant, got: {result:?}"
    );
}

#[tokio::test]
async fn canvas_of_another_tenant_is_not_readable() {
    let app_state = init_test_app_state("canvas").await;
    let first = create_tenant_with_owner(&app_state, "first").await;
    let second = create_tenant_with_owner(&app_state, "second").await;
    let canvas_id = insert_canvas(
        &app_state,
        first.tenant_id,
        "First canvas",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;

    let first_context = tenant_context_for(&app_state, &first).await;
    let second_context = tenant_context_for(&app_state, &second).await;

    assert!(
        crate::canvas::service::get_canvas(app_state.clone(), &first_context, canvas_id)
            .await
            .is_ok()
    );

    assert_unreachable(
        crate::canvas::service::get_canvas(app_state.clone(), &second_context, canvas_id).await,
        "canvas",
    );
}

#[tokio::test]
async fn canvas_list_excludes_other_tenants() {
    let app_state = init_test_app_state("canvas_list").await;
    let first = create_tenant_with_owner(&app_state, "first").await;
    let second = create_tenant_with_owner(&app_state, "second").await;
    insert_canvas(
        &app_state,
        first.tenant_id,
        "first tenant canvas",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;
    insert_canvas(
        &app_state,
        second.tenant_id,
        "second tenant canvas",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;

    let first_context = tenant_context_for(&app_state, &first).await;
    let page = crate::canvas::service::get_all_canvases(app_state.clone(), &first_context, 0, 50)
        .await
        .unwrap();

    assert_eq!(
        page.total_items, 1,
        "list must count only the requesting tenant"
    );
    assert_eq!(page.data.len(), 1);
    assert_eq!(page.data[0].name, "first tenant canvas");
}

#[tokio::test]
async fn notebook_of_another_tenant_is_not_readable() {
    let app_state = init_test_app_state("notebook").await;
    let first = create_tenant_with_owner(&app_state, "first").await;
    let second = create_tenant_with_owner(&app_state, "second").await;
    let canvas_id = insert_canvas(
        &app_state,
        first.tenant_id,
        "First canvas",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;
    let notebook_id = insert_notebook(&app_state, first.tenant_id, canvas_id, "Notebook").await;

    let first_context = tenant_context_for(&app_state, &first).await;
    let second_context = tenant_context_for(&app_state, &second).await;

    assert!(
        crate::notebook::service::get_notebook(app_state.clone(), &first_context, notebook_id)
            .await
            .is_ok()
    );

    assert_unreachable(
        crate::notebook::service::get_notebook(app_state.clone(), &second_context, notebook_id)
            .await,
        "notebook",
    );
}

#[tokio::test]
async fn cell_of_another_tenant_is_not_readable() {
    let app_state = init_test_app_state("cell").await;
    let first = create_tenant_with_owner(&app_state, "first").await;
    let second = create_tenant_with_owner(&app_state, "second").await;
    let canvas_id = insert_canvas(
        &app_state,
        first.tenant_id,
        "First canvas",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;
    let notebook_id = insert_notebook(&app_state, first.tenant_id, canvas_id, "Notebook").await;

    let cell_id = Uuid::new_v4();
    entity::cell::ActiveModel {
        id: Set(cell_id),
        tenant_id: Set(first.tenant_id),
        notebook_id: Set(notebook_id),
        connection_id: Set(None),
        display_order: Set(0),
        name: Set(Some("First cell".to_string())),
        content: Set(None),
        cell_type: Set("SQL".to_string()),
        status: Set("ACTIVE".to_string()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();

    let first_context = tenant_context_for(&app_state, &first).await;
    let second_context = tenant_context_for(&app_state, &second).await;

    assert!(
        crate::cell::service::get_cell(app_state.clone(), &first_context, cell_id)
            .await
            .is_ok()
    );

    assert_unreachable(
        crate::cell::service::get_cell(app_state.clone(), &second_context, cell_id).await,
        "cell",
    );
}

#[tokio::test]
async fn dashboard_of_another_tenant_is_not_readable() {
    let app_state = init_test_app_state("dashboard").await;
    let first = create_tenant_with_owner(&app_state, "first").await;
    let second = create_tenant_with_owner(&app_state, "second").await;
    let canvas_id = insert_canvas(
        &app_state,
        first.tenant_id,
        "First canvas",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;

    let dashboard_id = insert_dashboard(&app_state, first.tenant_id, canvas_id, "Dashboard").await;

    let first_context = tenant_context_for(&app_state, &first).await;
    let second_context = tenant_context_for(&app_state, &second).await;

    assert!(crate::dashboard::service::get_dashboard(
        app_state.clone(),
        &first_context,
        dashboard_id
    )
    .await
    .is_ok());

    assert_unreachable(
        crate::dashboard::service::get_dashboard(app_state.clone(), &second_context, dashboard_id)
            .await,
        "dashboard",
    );
}

#[tokio::test]
async fn connection_of_another_tenant_is_not_readable() {
    let app_state = init_test_app_state("connection").await;
    let first = create_tenant_with_owner(&app_state, "first").await;
    let second = create_tenant_with_owner(&app_state, "second").await;
    let connection_id = insert_connection(
        &app_state,
        first.tenant_id,
        "Connection",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;

    let first_context = tenant_context_for(&app_state, &first).await;
    let second_context = tenant_context_for(&app_state, &second).await;

    assert!(crate::connection::service::get_connection(
        app_state.clone(),
        &first_context,
        connection_id
    )
    .await
    .is_ok());

    assert_unreachable(
        crate::connection::service::get_connection(
            app_state.clone(),
            &second_context,
            connection_id,
        )
        .await,
        "connection",
    );
}

#[tokio::test]
async fn conversation_of_another_tenant_is_not_readable() {
    let app_state = init_test_app_state("conversation").await;
    let first = create_tenant_with_owner(&app_state, "first").await;
    let second = create_tenant_with_owner(&app_state, "second").await;
    let connection_id = insert_connection(
        &app_state,
        first.tenant_id,
        "Connection",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;
    let conversation_id = insert_conversation(
        &app_state,
        first.tenant_id,
        "Conversation",
        Uuid::new_v4(),
        Visibility::Tenant,
        connection_id,
    )
    .await;

    let first_context = tenant_context_for(&app_state, &first).await;
    let second_context = tenant_context_for(&app_state, &second).await;

    assert!(
        crate::conversational_analytics::conversation::service::get_conversation(
            app_state.clone(),
            &first_context,
            conversation_id
        )
        .await
        .is_ok()
    );

    assert_unreachable(
        crate::conversational_analytics::conversation::service::get_conversation(
            app_state.clone(),
            &second_context,
            conversation_id,
        )
        .await,
        "conversation",
    );
}

#[tokio::test]
async fn suggested_question_of_another_tenant_is_not_visible() {
    let app_state = init_test_app_state("suggested_question").await;
    let first = create_tenant_with_owner(&app_state, "first").await;
    let second = create_tenant_with_owner(&app_state, "second").await;
    let connection_id = insert_connection(
        &app_state,
        first.tenant_id,
        "Connection",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;

    entity::suggested_question::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(first.tenant_id),
        connection_id: Set(connection_id),
        question_text: Set("How many orders?".to_string()),
        generated_at: Set(time_utils::now_utc_into()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();

    let first_context = tenant_context_for(&app_state, &first).await;
    let second_context = tenant_context_for(&app_state, &second).await;

    let owner_view = crate::suggested_question::service::list_for_connection(
        &app_state,
        &first_context,
        connection_id,
    )
    .await
    .unwrap();
    assert_eq!(owner_view.len(), 1);

    assert_unreachable(
        crate::suggested_question::service::list_for_connection(
            &app_state,
            &second_context,
            connection_id,
        )
        .await,
        "suggested questions",
    );
}

#[tokio::test]
async fn ai_summary_of_another_tenant_is_not_visible() {
    use crate::ai_summary::constants::SummaryEntityType;

    let app_state = init_test_app_state("ai_summary").await;
    let first = create_tenant_with_owner(&app_state, "first").await;
    let second = create_tenant_with_owner(&app_state, "second").await;
    let canvas_id = insert_canvas(
        &app_state,
        first.tenant_id,
        "First canvas",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;
    let dashboard_id = insert_dashboard(&app_state, first.tenant_id, canvas_id, "Dashboard").await;

    entity::ai_summary::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(first.tenant_id),
        entity_type: Set(SummaryEntityType::Dashboard.to_string()),
        entity_id: Set(dashboard_id),
        status: Set("READY".to_string()),
        summary_text: Set(Some("secret summary".to_string())),
        error_message: Set(None),
        summarized_entity_count: Set(None),
        user_prompt: Set(None),
        user_prompt_used: Set(None),
        generated_at: Set(None),
        requested_at: Set(time_utils::now_utc_into()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();

    let first_context = tenant_context_for(&app_state, &first).await;
    let second_context = tenant_context_for(&app_state, &second).await;

    let owner_view = crate::ai_summary::service::get_dashboard_summaries(
        &app_state,
        &first_context,
        dashboard_id,
    )
    .await
    .unwrap();
    assert_eq!(
        owner_view
            .dashboard
            .expect("owner must see the dashboard summary")
            .summary_text,
        Some("secret summary".to_string())
    );

    assert_unreachable(
        crate::ai_summary::service::get_dashboard_summaries(
            &app_state,
            &second_context,
            dashboard_id,
        )
        .await,
        "ai summary",
    );
}

#[tokio::test]
async fn non_member_cannot_resolve_tenant_context() {
    let app_state = init_test_app_state("non_member").await;
    let first = create_tenant_with_owner(&app_state, "first").await;
    let second = create_tenant_with_owner(&app_state, "second").await;

    let result = crate::authorization::service::resolve_tenant_context(
        &app_state,
        second.user(),
        first.tenant_id,
    )
    .await;

    assert!(matches!(result, Err(TenantContextError::NotAMember)));
}

#[tokio::test]
async fn suspended_member_cannot_resolve_tenant_context() {
    let app_state = init_test_app_state("suspended").await;
    let fixture = create_tenant_with_owner(&app_state, "suspended").await;

    let membership = crate::authorization::service::find_membership(
        &app_state,
        fixture.tenant_id,
        fixture.user_id,
    )
    .await
    .unwrap()
    .unwrap();

    let mut membership_active: entity::tenant_membership::ActiveModel = membership.into();
    membership_active.status = Set("SUSPENDED".to_string());
    membership_active
        .update(&app_state.database_connection)
        .await
        .unwrap();

    let result = crate::authorization::service::resolve_tenant_context(
        &app_state,
        fixture.user(),
        fixture.tenant_id,
    )
    .await;

    assert!(matches!(
        result,
        Err(TenantContextError::MembershipSuspended)
    ));
}

#[tokio::test]
async fn user_belonging_to_two_tenants_is_scoped_to_each_separately() {
    let app_state = init_test_app_state("both_tenants").await;
    let first = create_tenant_with_owner(&app_state, "first").await;
    let second = create_tenant_with_owner(&app_state, "second").await;

    crate::tenant::membership::service::add_member_with_role_name(
        &app_state,
        &second.scope(),
        first.user(),
        SystemRole::Owner.as_str(),
    )
    .await
    .unwrap();

    insert_canvas(
        &app_state,
        first.tenant_id,
        "first canvas",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;
    insert_canvas(
        &app_state,
        second.tenant_id,
        "second canvas",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;

    let shared_user = first.user();
    for (tenant_id, expected_name) in [
        (first.tenant_id, "first canvas"),
        (second.tenant_id, "second canvas"),
    ] {
        let context = crate::authorization::service::resolve_tenant_context(
            &app_state,
            shared_user,
            tenant_id,
        )
        .await
        .unwrap();
        let page = crate::canvas::service::get_all_canvases(app_state.clone(), &context, 0, 50)
            .await
            .unwrap();
        assert_eq!(page.total_items, 1);
        assert_eq!(page.data[0].name, expected_name);
    }
}

#[tokio::test]
async fn tenant_context_carries_the_roles_permissions() {
    let app_state = init_test_app_state("permissions").await;
    let fixture = create_tenant_with_owner(&app_state, "owner").await;
    let context = tenant_context_for(&app_state, &fixture).await;

    use crate::authorization::constants::Permission;
    assert!(context.permissions().contains(Permission::CanvasRead));
    assert!(context.permissions().contains(Permission::TenantDelete));
    assert_eq!(context.user_id().get_uuid(), fixture.user_id);
    assert_eq!(context.tenant_id().get_uuid(), fixture.tenant_id);
}
