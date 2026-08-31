use crate::authorization::constants::{
    AccessLevel, Action, PrincipalType, ResourceType, SystemRole, Visibility,
};
use crate::authorization::dto::{AuthorizationSubject, TenantContext, TenantId};
use crate::authorization::errors::AuthorizationError;
use crate::common::time_utils;
use crate::common::AppState;
use crate::entity;
use crate::tests::common::app_state::init_test_app_state;
use crate::tests::common::resources::{
    insert_canvas, insert_cell, insert_connection, insert_notebook,
};
use crate::tests::common::tenancy::{context_with_role, create_tenant};
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

fn canvas_subject(
    tenant_id: Uuid,
    owner_user_id: Uuid,
    visibility: Visibility,
) -> AuthorizationSubject {
    AuthorizationSubject::new(
        TenantId::from_system_unchecked(tenant_id),
        ResourceType::Canvas,
        Uuid::new_v4(),
        owner_user_id,
        visibility,
    )
}

async fn grant(
    app_state: &AppState,
    tenant_id: Uuid,
    subject: &AuthorizationSubject,
    principal_id: Uuid,
    access_level: AccessLevel,
) {
    entity::resource_grant::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        resource_type: Set(subject.resource_type().as_str().to_string()),
        resource_id: Set(subject.resource_id()),
        principal_type: Set(PrincipalType::User.as_str().to_string()),
        principal_id: Set(principal_id),
        access_level: Set(access_level.as_str().to_string()),
        granted_by_user_id: Set(Uuid::new_v4()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();
}

async fn allowed(
    app_state: &AppState,
    context: &TenantContext,
    subject: &AuthorizationSubject,
    action: Action,
) -> bool {
    crate::authorization::service::require_access(app_state, context, subject, action)
        .await
        .is_ok()
}

#[tokio::test]
async fn owner_can_read_and_write_own_private_canvas() {
    let app_state = init_test_app_state("owner_private").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let context = context_with_role(&app_state, tenant_id, SystemRole::Member, "member").await;
    let subject = canvas_subject(tenant_id, context.user_id().get_uuid(), Visibility::Private);

    assert!(allowed(&app_state, &context, &subject, Action::Read).await);
    assert!(allowed(&app_state, &context, &subject, Action::Write).await);
    assert!(allowed(&app_state, &context, &subject, Action::Share).await);
}

#[tokio::test]
async fn member_cannot_reach_another_members_private_canvas() {
    let app_state = init_test_app_state("member_private").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let context = context_with_role(&app_state, tenant_id, SystemRole::Member, "member").await;
    let subject = canvas_subject(tenant_id, Uuid::new_v4(), Visibility::Private);

    assert!(!allowed(&app_state, &context, &subject, Action::Read).await);
    assert!(!allowed(&app_state, &context, &subject, Action::Write).await);
}

#[tokio::test]
async fn admin_can_read_private_canvas_it_does_not_own() {
    let app_state = init_test_app_state("admin_private").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let context = context_with_role(&app_state, tenant_id, SystemRole::Admin, "admin").await;
    let subject = canvas_subject(tenant_id, Uuid::new_v4(), Visibility::Private);

    assert!(allowed(&app_state, &context, &subject, Action::Read).await);
    assert!(allowed(&app_state, &context, &subject, Action::Write).await);
}

#[tokio::test]
async fn tenant_visibility_grants_read_but_not_write() {
    let app_state = init_test_app_state("tenant_visibility").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let context = context_with_role(&app_state, tenant_id, SystemRole::Member, "member").await;
    let subject = canvas_subject(tenant_id, Uuid::new_v4(), Visibility::Tenant);

    assert!(allowed(&app_state, &context, &subject, Action::Read).await);
    assert!(
        !allowed(&app_state, &context, &subject, Action::Write).await,
        "tenant-visible must not mean tenant-editable"
    );
    assert!(!allowed(&app_state, &context, &subject, Action::Delete).await);
}

#[tokio::test]
async fn viewer_role_cannot_write_even_what_it_can_read() {
    let app_state = init_test_app_state("viewer").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let context = context_with_role(&app_state, tenant_id, SystemRole::Viewer, "viewer").await;
    let subject = canvas_subject(tenant_id, Uuid::new_v4(), Visibility::Tenant);

    assert!(allowed(&app_state, &context, &subject, Action::Read).await);
    let write = crate::authorization::service::require_access(
        &app_state,
        &context,
        &subject,
        Action::Write,
    )
    .await;
    assert!(
        matches!(write, Err(AuthorizationError::MissingPermission(_))),
        "viewer must fail the capability gate, not the reachability gate"
    );
}

#[tokio::test]
async fn viewer_grant_allows_read_only_and_editor_grant_allows_write() {
    let app_state = init_test_app_state("grants").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let reader = context_with_role(&app_state, tenant_id, SystemRole::Member, "reader").await;
    let editor = context_with_role(&app_state, tenant_id, SystemRole::Member, "editor").await;

    let read_subject = canvas_subject(tenant_id, Uuid::new_v4(), Visibility::Private);
    grant(
        &app_state,
        tenant_id,
        &read_subject,
        reader.user_id().get_uuid(),
        AccessLevel::Viewer,
    )
    .await;

    let write_subject = canvas_subject(tenant_id, Uuid::new_v4(), Visibility::Private);
    grant(
        &app_state,
        tenant_id,
        &write_subject,
        editor.user_id().get_uuid(),
        AccessLevel::Editor,
    )
    .await;

    assert!(allowed(&app_state, &reader, &read_subject, Action::Read).await);
    assert!(!allowed(&app_state, &reader, &read_subject, Action::Write).await);

    assert!(allowed(&app_state, &editor, &write_subject, Action::Read).await);
    assert!(allowed(&app_state, &editor, &write_subject, Action::Write).await);
    assert!(
        !allowed(&app_state, &editor, &write_subject, Action::Share).await,
        "an editor grantee must not be able to re-share"
    );
}

#[tokio::test]
async fn subject_from_another_tenant_is_rejected_before_any_gate() {
    let app_state = init_test_app_state("cross_tenant").await;
    let first_tenant = create_tenant(&app_state, "first").await;
    let second_tenant = create_tenant(&app_state, "second").await;
    let context = context_with_role(&app_state, first_tenant, SystemRole::Owner, "owner").await;

    let subject = canvas_subject(
        second_tenant,
        context.user_id().get_uuid(),
        Visibility::Tenant,
    );

    let result =
        crate::authorization::service::require_access(&app_state, &context, &subject, Action::Read)
            .await;
    assert!(
        matches!(result, Err(AuthorizationError::WrongTenant)),
        "owning the resource must not override tenant isolation"
    );
}

async fn visible_names(app_state: &AppState, context: &TenantContext) -> Vec<String> {
    let page = crate::canvas::service::get_all_canvases(app_state.clone(), context, 0, 50)
        .await
        .unwrap();
    let mut names: Vec<String> = page.data.into_iter().map(|canvas| canvas.name).collect();
    names.sort();
    names
}

#[tokio::test]
async fn list_returns_exactly_what_the_gate_would_allow() {
    let app_state = init_test_app_state("list_reachability").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let member = context_with_role(&app_state, tenant_id, SystemRole::Member, "member").await;
    let member_user_id = member.user_id().get_uuid();

    insert_canvas(
        &app_state,
        tenant_id,
        "own private",
        member_user_id,
        Visibility::Private,
    )
    .await;
    insert_canvas(
        &app_state,
        tenant_id,
        "tenant visible",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;
    insert_canvas(
        &app_state,
        tenant_id,
        "someone elses private",
        Uuid::new_v4(),
        Visibility::Private,
    )
    .await;

    let granted_canvas_id = insert_canvas(
        &app_state,
        tenant_id,
        "granted private",
        Uuid::new_v4(),
        Visibility::Private,
    )
    .await;
    entity::resource_grant::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        resource_type: Set(ResourceType::Canvas.as_str().to_string()),
        resource_id: Set(granted_canvas_id),
        principal_type: Set(PrincipalType::User.as_str().to_string()),
        principal_id: Set(member_user_id),
        access_level: Set(AccessLevel::Viewer.as_str().to_string()),
        granted_by_user_id: Set(Uuid::new_v4()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();

    assert_eq!(
        visible_names(&app_state, &member).await,
        vec![
            "granted private".to_string(),
            "own private".to_string(),
            "tenant visible".to_string()
        ],
        "the list must show owned, tenant-visible and granted canvases only"
    );
}

#[tokio::test]
async fn list_shows_everything_to_a_role_that_can_read_private() {
    let app_state = init_test_app_state("list_admin").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let admin = context_with_role(&app_state, tenant_id, SystemRole::Admin, "admin").await;

    insert_canvas(
        &app_state,
        tenant_id,
        "a private",
        Uuid::new_v4(),
        Visibility::Private,
    )
    .await;
    insert_canvas(
        &app_state,
        tenant_id,
        "b tenant",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;

    assert_eq!(
        visible_names(&app_state, &admin).await,
        vec!["a private".to_string(), "b tenant".to_string()]
    );
}

#[tokio::test]
async fn list_never_crosses_tenants() {
    let app_state = init_test_app_state("list_cross_tenant").await;
    let first_tenant = create_tenant(&app_state, "first").await;
    let second_tenant = create_tenant(&app_state, "second").await;
    let member = context_with_role(&app_state, first_tenant, SystemRole::Member, "member").await;
    let admin = context_with_role(&app_state, first_tenant, SystemRole::Admin, "admin").await;

    insert_canvas(
        &app_state,
        first_tenant,
        "mine",
        member.user_id().get_uuid(),
        Visibility::Private,
    )
    .await;
    insert_canvas(
        &app_state,
        second_tenant,
        "theirs",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;

    assert_eq!(
        visible_names(&app_state, &member).await,
        vec!["mine".to_string()]
    );
    assert_eq!(
        visible_names(&app_state, &admin).await,
        vec!["mine".to_string()],
        "read_private must not lift the tenant boundary"
    );
}

#[tokio::test]
async fn capabilities_reflect_what_the_gates_allow() {
    use crate::authorization::service::resolve_capabilities;

    let app_state = init_test_app_state("capabilities").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let member = context_with_role(&app_state, tenant_id, SystemRole::Member, "member").await;
    let viewer = context_with_role(&app_state, tenant_id, SystemRole::Viewer, "viewer").await;

    let owned = canvas_subject(tenant_id, member.user_id().get_uuid(), Visibility::Private);
    let owned_capabilities = resolve_capabilities(&app_state, &member, &owned).await;
    assert!(owned_capabilities.can_edit);
    assert!(owned_capabilities.can_delete);
    assert!(owned_capabilities.can_share);

    let someone_elses = canvas_subject(tenant_id, Uuid::new_v4(), Visibility::Tenant);
    let readable_capabilities = resolve_capabilities(&app_state, &member, &someone_elses).await;
    assert!(
        !readable_capabilities.can_edit,
        "tenant-visible is readable but not editable"
    );
    assert!(!readable_capabilities.can_delete);
    assert!(!readable_capabilities.can_share);

    let viewer_capabilities = resolve_capabilities(&app_state, &viewer, &someone_elses).await;
    assert!(!viewer_capabilities.can_edit);
    assert!(!viewer_capabilities.can_delete);
    assert!(!viewer_capabilities.can_share);
}

#[tokio::test]
async fn a_child_resource_inherits_its_roots_decision() {
    let app_state = init_test_app_state("child_inherits").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let owner = context_with_role(&app_state, tenant_id, SystemRole::Member, "owner").await;
    let other = context_with_role(&app_state, tenant_id, SystemRole::Member, "other").await;

    let canvas_id = insert_canvas(
        &app_state,
        tenant_id,
        "private canvas",
        owner.user_id().get_uuid(),
        Visibility::Private,
    )
    .await;

    let notebook_id = Uuid::new_v4();
    entity::notebook::ActiveModel {
        id: Set(notebook_id),
        tenant_id: Set(tenant_id),
        canvas_id: Set(canvas_id),
        name: Set("Notebook".to_string()),
        description: Set(None),
        status: Set("ACTIVE".to_string()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();

    let owner_view =
        crate::notebook::service::authorize_notebook(&app_state, &owner, notebook_id, Action::Read)
            .await;
    assert!(
        owner_view.is_ok(),
        "the canvas owner must reach a notebook inside it"
    );

    let other_view =
        crate::notebook::service::authorize_notebook(&app_state, &other, notebook_id, Action::Read)
            .await;
    assert!(
        other_view.is_err(),
        "a notebook must not be reachable when its canvas is not"
    );
}

async fn visible_notebook_names(app_state: &AppState, context: &TenantContext) -> Vec<String> {
    let page = crate::notebook::service::get_all_notebooks(app_state.clone(), context, 0, 50)
        .await
        .unwrap();
    let mut names: Vec<String> = page
        .data
        .into_iter()
        .map(|notebook| notebook.name)
        .collect();
    names.sort();
    names
}

async fn visible_connection_names(app_state: &AppState, context: &TenantContext) -> Vec<String> {
    let connections = crate::connection::service::get_all_connections(app_state.clone(), context)
        .await
        .unwrap();
    let mut names: Vec<String> = connections
        .into_iter()
        .map(|connection| connection.name)
        .collect();
    names.sort();
    names
}

#[tokio::test]
async fn notebook_list_inherits_the_reachability_of_its_canvas() {
    let app_state = init_test_app_state("notebook_list_reachability").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let member = context_with_role(&app_state, tenant_id, SystemRole::Member, "member").await;
    let member_user_id = member.user_id().get_uuid();

    let own_canvas_id = insert_canvas(
        &app_state,
        tenant_id,
        "own canvas",
        member_user_id,
        Visibility::Private,
    )
    .await;
    let tenant_canvas_id = insert_canvas(
        &app_state,
        tenant_id,
        "tenant canvas",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;
    let foreign_canvas_id = insert_canvas(
        &app_state,
        tenant_id,
        "someone else's canvas",
        Uuid::new_v4(),
        Visibility::Private,
    )
    .await;
    let granted_canvas_id = insert_canvas(
        &app_state,
        tenant_id,
        "granted canvas",
        Uuid::new_v4(),
        Visibility::Private,
    )
    .await;
    entity::resource_grant::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        resource_type: Set(ResourceType::Canvas.as_str().to_string()),
        resource_id: Set(granted_canvas_id),
        principal_type: Set(PrincipalType::User.as_str().to_string()),
        principal_id: Set(member_user_id),
        access_level: Set(AccessLevel::Viewer.as_str().to_string()),
        granted_by_user_id: Set(Uuid::new_v4()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();

    insert_notebook(&app_state, tenant_id, own_canvas_id, "a own").await;
    insert_notebook(&app_state, tenant_id, tenant_canvas_id, "b tenant").await;
    insert_notebook(&app_state, tenant_id, foreign_canvas_id, "c foreign").await;
    insert_notebook(&app_state, tenant_id, granted_canvas_id, "d granted").await;

    assert_eq!(
        visible_notebook_names(&app_state, &member).await,
        vec![
            "a own".to_string(),
            "b tenant".to_string(),
            "d granted".to_string()
        ],
        "a notebook must be listed only when its canvas is reachable"
    );

    let admin = context_with_role(&app_state, tenant_id, SystemRole::Admin, "admin").await;
    assert_eq!(
        visible_notebook_names(&app_state, &admin).await,
        vec![
            "a own".to_string(),
            "b tenant".to_string(),
            "c foreign".to_string(),
            "d granted".to_string()
        ],
        "read_private on the parent canvas must reach every notebook"
    );
}

#[tokio::test]
async fn notebook_list_never_crosses_tenants() {
    let app_state = init_test_app_state("notebook_list_cross_tenant").await;
    let first_tenant = create_tenant(&app_state, "first").await;
    let second_tenant = create_tenant(&app_state, "second").await;
    let admin = context_with_role(&app_state, first_tenant, SystemRole::Admin, "admin").await;

    let mine = insert_canvas(
        &app_state,
        first_tenant,
        "mine",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;
    let theirs = insert_canvas(
        &app_state,
        second_tenant,
        "theirs",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;
    insert_notebook(&app_state, first_tenant, mine, "mine").await;
    insert_notebook(&app_state, second_tenant, theirs, "theirs").await;

    assert_eq!(
        visible_notebook_names(&app_state, &admin).await,
        vec!["mine".to_string()],
        "read_private must not lift the tenant boundary"
    );
}

#[tokio::test]
async fn connection_list_returns_exactly_what_the_gate_would_allow() {
    let app_state = init_test_app_state("connection_list_reachability").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let member = context_with_role(&app_state, tenant_id, SystemRole::Member, "member").await;
    let member_user_id = member.user_id().get_uuid();

    insert_connection(
        &app_state,
        tenant_id,
        "a own private",
        member_user_id,
        Visibility::Private,
    )
    .await;
    insert_connection(
        &app_state,
        tenant_id,
        "b tenant",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;
    insert_connection(
        &app_state,
        tenant_id,
        "c foreign private",
        Uuid::new_v4(),
        Visibility::Private,
    )
    .await;
    let granted_connection_id = insert_connection(
        &app_state,
        tenant_id,
        "d granted private",
        Uuid::new_v4(),
        Visibility::Private,
    )
    .await;
    entity::resource_grant::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        resource_type: Set(ResourceType::Connection.as_str().to_string()),
        resource_id: Set(granted_connection_id),
        principal_type: Set(PrincipalType::User.as_str().to_string()),
        principal_id: Set(member_user_id),
        access_level: Set(AccessLevel::Viewer.as_str().to_string()),
        granted_by_user_id: Set(Uuid::new_v4()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();

    assert_eq!(
        visible_connection_names(&app_state, &member).await,
        vec![
            "a own private".to_string(),
            "b tenant".to_string(),
            "d granted private".to_string()
        ]
    );

    let admin = context_with_role(&app_state, tenant_id, SystemRole::Admin, "admin").await;
    assert_eq!(
        visible_connection_names(&app_state, &admin).await,
        vec![
            "a own private".to_string(),
            "b tenant".to_string(),
            "c foreign private".to_string(),
            "d granted private".to_string()
        ]
    );
}

#[tokio::test]
async fn connection_list_never_crosses_tenants() {
    let app_state = init_test_app_state("connection_list_cross_tenant").await;
    let first_tenant = create_tenant(&app_state, "first").await;
    let second_tenant = create_tenant(&app_state, "second").await;
    let admin = context_with_role(&app_state, first_tenant, SystemRole::Admin, "admin").await;

    insert_connection(
        &app_state,
        first_tenant,
        "mine",
        Uuid::new_v4(),
        Visibility::Private,
    )
    .await;
    insert_connection(
        &app_state,
        second_tenant,
        "theirs",
        Uuid::new_v4(),
        Visibility::Tenant,
    )
    .await;

    assert_eq!(
        visible_connection_names(&app_state, &admin).await,
        vec!["mine".to_string()],
        "read_private must not lift the tenant boundary"
    );
}

#[tokio::test]
async fn search_does_not_surface_cells_of_unreachable_canvases() {
    let app_state = init_test_app_state("search_cell_reachability").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let member = context_with_role(&app_state, tenant_id, SystemRole::Member, "member").await;

    let own_canvas_id = insert_canvas(
        &app_state,
        tenant_id,
        "own canvas",
        member.user_id().get_uuid(),
        Visibility::Private,
    )
    .await;
    let foreign_canvas_id = insert_canvas(
        &app_state,
        tenant_id,
        "foreign canvas",
        Uuid::new_v4(),
        Visibility::Private,
    )
    .await;

    let own_notebook_id = Uuid::new_v4();
    let foreign_notebook_id = Uuid::new_v4();
    for (notebook_id, canvas_id) in [
        (own_notebook_id, own_canvas_id),
        (foreign_notebook_id, foreign_canvas_id),
    ] {
        entity::notebook::ActiveModel {
            id: Set(notebook_id),
            tenant_id: Set(tenant_id),
            canvas_id: Set(canvas_id),
            name: Set("notebook".to_string()),
            description: Set(None),
            status: Set("ACTIVE".to_string()),
            created_at: Set(time_utils::now_utc_into()),
            updated_at: Set(time_utils::now_utc_into()),
        }
        .insert(&app_state.database_connection)
        .await
        .unwrap();
    }

    insert_cell(&app_state, tenant_id, own_notebook_id, "reachable query").await;
    insert_cell(&app_state, tenant_id, foreign_notebook_id, "private query").await;

    let found = crate::cell::service::search_cells_by_name_and_type(
        &app_state,
        &member,
        "query",
        crate::cell::constants::CellType::Sql,
        50,
    )
    .await
    .unwrap();
    let names: Vec<String> = found.into_iter().filter_map(|cell| cell.name).collect();
    assert_eq!(
        names,
        vec!["reachable query".to_string()],
        "search must not reveal cells living in a canvas the searcher cannot reach"
    );

    let admin = context_with_role(&app_state, tenant_id, SystemRole::Admin, "admin").await;
    let found_by_admin = crate::cell::service::search_cells_by_name_and_type(
        &app_state,
        &admin,
        "query",
        crate::cell::constants::CellType::Sql,
        50,
    )
    .await
    .unwrap();
    assert_eq!(
        found_by_admin.len(),
        2,
        "read_private must reach cells in every canvas of the tenant"
    );
}

#[tokio::test]
async fn search_does_not_surface_unreachable_conversations() {
    let app_state = init_test_app_state("search_conversation_reachability").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let member = context_with_role(&app_state, tenant_id, SystemRole::Member, "member").await;

    for (name, owner_user_id, visibility) in [
        ("mine", member.user_id().get_uuid(), Visibility::Private),
        ("theirs", Uuid::new_v4(), Visibility::Private),
        ("shared", Uuid::new_v4(), Visibility::Tenant),
    ] {
        entity::conversation::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            owner_user_id: Set(owner_user_id),
            visibility: Set(visibility.as_str().to_string()),
            connection_id: Set(Uuid::new_v4()),
            name: Set(name.to_string()),
            status: Set("ACTIVE".to_string()),
            created_at: Set(time_utils::now_utc_into()),
            updated_at: Set(time_utils::now_utc_into()),
        }
        .insert(&app_state.database_connection)
        .await
        .unwrap();
    }

    let found =
        crate::conversational_analytics::conversation::service::search_conversations_by_name(
            &app_state, &member, None, 50,
        )
        .await
        .unwrap();
    let mut names: Vec<String> = found
        .into_iter()
        .map(|conversation| conversation.name)
        .collect();
    names.sort();
    assert_eq!(names, vec!["mine".to_string(), "shared".to_string()]);
}
