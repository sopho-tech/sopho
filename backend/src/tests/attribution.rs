use crate::ai::dto::{CanvasOp, CanvasPlan};
use crate::ai_configuration::dto::Provider;
use crate::attribution::dto::{AttributedUserDto, AttributionDto};
use crate::authorization::constants::{ActorKind, ResourceType, SystemRole, Visibility};
use crate::authorization::dto::{Actor, ResourceOwnership, TenantContext, UserId};
use crate::canvas::dto::{CanvasAuthorship, CreateCanvasDto};
use crate::cell::constants::{CellDisplayOrderMovement, CellType};
use crate::cell::dto::{CellDto, CreateCellDto, ReorderCellDto};
use crate::common::constants::CreatedByFilter;
use crate::common::cryptography_utils;
use crate::common::time_utils;
use crate::common::{AppState, CreatedByQuery};
use crate::conversational_analytics::conversation::constants::MessageStatus;
use crate::conversational_analytics::conversation::dto::{
    AppendUserMessageDto, CreateConversationDto, ListConversationsQuery,
};
use crate::conversational_analytics::message::constants::Sender;
use crate::conversational_analytics::message::dto::CreateConversationMessageDto;
use crate::conversational_analytics::segment::MessageSegment;
use crate::dashboard::DashboardDto;
use crate::dashboard::constants::DashboardStatus;
use crate::entity;
use crate::grant::dto::UpdateVisibilityDto;
use crate::tenant::invitation::dto::CreateInvitationDto;
use crate::tests::common::app_state::{
    init_test_app_state, init_test_app_state_with_single_sign_on,
};
use crate::tests::common::resources::insert_connection;
use crate::tests::common::tenancy::{
    context_with_role, create_tenant, create_tenant_with_owner, human_actor, insert_user,
    resolve_context,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbBackend, DbErr, EntityTrait, QueryFilter,
    Set, Statement, TransactionTrait,
};
use uuid::Uuid;

const TEST_ADMIN_EMAIL: &str = "admin@test.local";
const DEMO_CANVAS_NAME: &str = "Ecommerce Sales";
const DEMO_CONNECTION_NAME: &str = "Ecommerce Sales Demo";
const LIVE_AI_STATUS: &str = "LIVE";
const TEST_ADMIN_FULL_NAME: &str = "Test Admin";

const TABLES_IN_SCOPE: [&str; 14] = [
    "canvas",
    "notebook",
    "cell",
    "dashboard",
    "connection",
    "conversation",
    "conversation_message",
    "conversation_message_content",
    "ai_summary",
    "suggested_question",
    "ai_configuration",
    "tenant",
    "tenant_membership",
    "role",
];

#[derive(Debug, PartialEq)]
struct RecordedActor {
    kind: String,
    user_id: Option<Uuid>,
}

impl From<Actor> for RecordedActor {
    fn from(actor: Actor) -> Self {
        RecordedActor {
            kind: actor.kind_name(),
            user_id: actor.user_id(),
        }
    }
}

trait Attributed {
    fn created_by(&self) -> RecordedActor;
    fn modified_by(&self) -> RecordedActor;
}

macro_rules! attributed {
    ($($model:path),+ $(,)?) => {
        $(impl Attributed for $model {
            fn created_by(&self) -> RecordedActor {
                RecordedActor {
                    kind: self.created_by_actor.clone(),
                    user_id: self.created_by_user_id,
                }
            }

            fn modified_by(&self) -> RecordedActor {
                RecordedActor {
                    kind: self.modified_by_actor.clone(),
                    user_id: self.modified_by_user_id,
                }
            }
        })+
    };
}

attributed!(
    entity::canvas::Model,
    entity::notebook::Model,
    entity::dashboard::Model,
    entity::cell::Model,
    entity::connection::Model,
    entity::conversation::Model,
    entity::conversation_message::Model,
    entity::tenant::Model,
    entity::role::Model,
    entity::tenant_membership::Model,
);

async fn find_canvas(app_state: &AppState, canvas_id: Uuid) -> entity::canvas::Model {
    entity::canvas::Entity::find_by_id(canvas_id)
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap()
}

async fn find_notebook_of(app_state: &AppState, canvas_id: Uuid) -> entity::notebook::Model {
    entity::notebook::Entity::find()
        .filter(entity::notebook::Column::CanvasId.eq(canvas_id))
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap()
}

async fn find_dashboard_of(app_state: &AppState, canvas_id: Uuid) -> entity::dashboard::Model {
    entity::dashboard::Entity::find()
        .filter(entity::dashboard::Column::CanvasId.eq(canvas_id))
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap()
}

async fn find_cells_of(app_state: &AppState, notebook_id: Uuid) -> Vec<entity::cell::Model> {
    entity::cell::Entity::find()
        .filter(entity::cell::Column::NotebookId.eq(notebook_id))
        .all(&app_state.database_connection)
        .await
        .unwrap()
}

async fn find_membership(
    app_state: &AppState,
    tenant_id: Uuid,
    user_id: Uuid,
) -> entity::tenant_membership::Model {
    entity::tenant_membership::Entity::find()
        .filter(entity::tenant_membership::Column::TenantId.eq(tenant_id))
        .filter(entity::tenant_membership::Column::UserId.eq(user_id))
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap()
}

async fn find_roles_of(app_state: &AppState, tenant_id: Uuid) -> Vec<entity::role::Model> {
    entity::role::Entity::find()
        .filter(entity::role::Column::TenantId.eq(tenant_id))
        .all(&app_state.database_connection)
        .await
        .unwrap()
}

async fn find_admin_user_id(app_state: &AppState) -> Uuid {
    crate::authentication::service::find_user_by_email(app_state, TEST_ADMIN_EMAIL)
        .await
        .unwrap()
        .id
}

async fn create_canvas_as(app_state: &AppState, context: &TenantContext, name: &str) -> Uuid {
    crate::canvas::service::create_canvas(
        app_state.clone(),
        context,
        context.user_id().get_uuid(),
        CreateCanvasDto {
            name: name.to_string(),
            description: None,
        },
    )
    .await
    .unwrap()
    .id
}

struct RawAttribution {
    actor: &'static str,
    user_id: Option<Uuid>,
}

async fn insert_canvas_attributed(
    app_state: &AppState,
    tenant_id: Uuid,
    created: &RawAttribution,
    modified: &RawAttribution,
) -> Result<entity::canvas::Model, DbErr> {
    entity::canvas::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        owner_user_id: Set(Uuid::new_v4()),
        visibility: Set(Visibility::Tenant.as_str().to_string()),
        name: Set("checked".to_string()),
        description: Set(None),
        status: Set("ACTIVE".to_string()),
        created_by_user_id: Set(created.user_id),
        created_by_actor: Set(created.actor.to_string()),
        modified_by_user_id: Set(modified.user_id),
        modified_by_actor: Set(modified.actor.to_string()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
}

#[tokio::test]
async fn every_table_in_scope_carries_attribution_columns() {
    let app_state = init_test_app_state("attribution_columns").await;

    for table in TABLES_IN_SCOPE {
        let columns = app_state
            .database_connection
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                format!("PRAGMA table_info({table})"),
            ))
            .await
            .unwrap();
        let not_null_by_column: std::collections::HashMap<String, i32> = columns
            .iter()
            .map(|column| {
                (
                    column.try_get::<String>("", "name").unwrap(),
                    column.try_get::<i32>("", "notnull").unwrap(),
                )
            })
            .collect();

        for (column, expected_not_null) in [
            ("created_by_user_id", 0),
            ("created_by_actor", 1),
            ("modified_by_user_id", 0),
            ("modified_by_actor", 1),
        ] {
            assert_eq!(
                not_null_by_column.get(column),
                Some(&expected_not_null),
                "{table}.{column} must exist with not-null = {expected_not_null}"
            );
        }
    }
}

#[tokio::test]
async fn the_database_refuses_unattributed_rows() {
    let app_state = init_test_app_state("attribution_check_constraints").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let person = Some(Uuid::new_v4());
    let valid = RawAttribution {
        actor: "HUMAN",
        user_id: person,
    };
    let invalid_pairs = [
        RawAttribution {
            actor: "HUMAN",
            user_id: None,
        },
        RawAttribution {
            actor: "AI",
            user_id: None,
        },
        RawAttribution {
            actor: "SYSTEM",
            user_id: person,
        },
        RawAttribution {
            actor: "ROBOT",
            user_id: person,
        },
    ];

    for invalid in &invalid_pairs {
        assert!(
            insert_canvas_attributed(&app_state, tenant_id, invalid, &valid)
                .await
                .is_err(),
            "created pair ({}, {:?}) must be refused",
            invalid.actor,
            invalid.user_id
        );
        assert!(
            insert_canvas_attributed(&app_state, tenant_id, &valid, invalid)
                .await
                .is_err(),
            "modified pair ({}, {:?}) must be refused",
            invalid.actor,
            invalid.user_id
        );
    }

    let system = RawAttribution {
        actor: "SYSTEM",
        user_id: None,
    };
    let ai = RawAttribution {
        actor: "AI",
        user_id: person,
    };
    insert_canvas_attributed(&app_state, tenant_id, &system, &system)
        .await
        .unwrap();
    insert_canvas_attributed(&app_state, tenant_id, &ai, &valid)
        .await
        .unwrap();
}

#[tokio::test]
async fn requests_stamp_the_person_who_made_them() {
    let app_state = init_test_app_state("attribution_requests").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let creator = context_with_role(&app_state, tenant_id, SystemRole::Member, "creator").await;
    let editor = context_with_role(&app_state, tenant_id, SystemRole::Admin, "editor").await;

    let own_canvas_id = create_canvas_as(&app_state, &editor, "own").await;
    let others_canvas_id = create_canvas_as(&app_state, &creator, "theirs").await;

    let others_notebook = find_notebook_of(&app_state, others_canvas_id).await;
    let others_dashboard = find_dashboard_of(&app_state, others_canvas_id).await;
    assert_eq!(others_notebook.created_by(), creator.actor().into());
    assert_eq!(others_dashboard.created_by(), creator.actor().into());

    for canvas_id in [own_canvas_id, others_canvas_id] {
        crate::canvas::service::update_canvas(
            app_state.clone(),
            &editor,
            canvas_id,
            CreateCanvasDto {
                name: "renamed".to_string(),
                description: None,
            },
        )
        .await
        .unwrap();
    }

    let own_canvas = find_canvas(&app_state, own_canvas_id).await;
    assert_eq!(own_canvas.created_by(), editor.actor().into());
    assert_eq!(own_canvas.modified_by(), editor.actor().into());

    let others_canvas = find_canvas(&app_state, others_canvas_id).await;
    assert_eq!(others_canvas.created_by(), creator.actor().into());
    assert_eq!(others_canvas.modified_by(), editor.actor().into());

    crate::grant::service::update_visibility(
        app_state.clone(),
        &creator,
        ResourceType::Canvas.as_str().to_string(),
        others_canvas_id,
        UpdateVisibilityDto {
            visibility: Visibility::Private.as_str().to_string(),
        },
    )
    .await
    .unwrap();
    let shared_canvas = find_canvas(&app_state, others_canvas_id).await;
    assert_eq!(shared_canvas.created_by(), creator.actor().into());
    assert_eq!(shared_canvas.modified_by(), creator.actor().into());
}

#[tokio::test]
async fn agent_work_is_stamped_for_the_person_who_prompted_it() {
    let app_state = init_test_app_state("attribution_agent").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let alice = context_with_role(&app_state, tenant_id, SystemRole::Member, "alice").await;
    let bob = context_with_role(&app_state, tenant_id, SystemRole::Member, "bob").await;
    let connection_id = insert_connection(
        &app_state,
        tenant_id,
        "warehouse",
        alice.user_id().get_uuid(),
        Visibility::Tenant,
    )
    .await;

    let summary = crate::canvas::service::apply_canvas_plan(
        &app_state,
        alice.tenant_id(),
        CanvasAuthorship {
            ownership: ResourceOwnership {
                owner_user_id: alice.user_id().get_uuid(),
                visibility: Visibility::Private,
            },
            actor: bob.ai_actor(),
        },
        connection_id,
        CanvasPlan {
            target_canvas_index: None,
            reasoning: String::new(),
            name: "Agent canvas".to_string(),
            description: String::new(),
            cells: Vec::new(),
        },
        None,
        vec![CanvasOp::Create {
            title: Some("Revenue".to_string()),
            sql: "select 1".to_string(),
            chart: None,
        }],
    )
    .await
    .unwrap();

    let canvas = find_canvas(&app_state, summary.canvas_id).await;
    assert_eq!(canvas.owner_user_id, alice.user_id().get_uuid());
    assert_eq!(canvas.created_by(), bob.ai_actor().into());
    assert_eq!(canvas.modified_by(), bob.ai_actor().into());

    let notebook = find_notebook_of(&app_state, canvas.id).await;
    assert_eq!(notebook.created_by(), bob.ai_actor().into());
    assert_eq!(
        find_dashboard_of(&app_state, canvas.id).await.created_by(),
        bob.ai_actor().into()
    );

    let cells = find_cells_of(&app_state, notebook.id).await;
    assert_eq!(cells.len(), 1);
    assert_eq!(cells[0].created_by(), bob.ai_actor().into());
    assert_eq!(cells[0].modified_by(), bob.ai_actor().into());
}

#[tokio::test]
async fn startup_work_is_stamped_as_the_system() {
    let app_state = init_test_app_state("attribution_startup").await;

    crate::initialize(&app_state).await.unwrap();

    let tenants = entity::tenant::Entity::find()
        .all(&app_state.database_connection)
        .await
        .unwrap();
    assert_eq!(tenants.len(), 1);
    let default_tenant = &tenants[0];
    assert_eq!(default_tenant.created_by(), Actor::system().into());
    assert_eq!(default_tenant.modified_by(), Actor::system().into());

    let roles = find_roles_of(&app_state, default_tenant.id).await;
    assert_eq!(roles.len(), SystemRole::all().len());
    for role in &roles {
        assert_eq!(
            role.created_by(),
            Actor::system().into(),
            "role {}",
            role.name
        );
    }

    let admin_user_id = find_admin_user_id(&app_state).await;
    let admin_membership = find_membership(&app_state, default_tenant.id, admin_user_id).await;
    assert_eq!(admin_membership.created_by(), Actor::system().into());

    let demo_connection = entity::connection::Entity::find()
        .filter(entity::connection::Column::Name.eq(DEMO_CONNECTION_NAME))
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(demo_connection.created_by(), Actor::system().into());

    let demo_canvas = entity::canvas::Entity::find()
        .filter(entity::canvas::Column::Name.eq(DEMO_CANVAS_NAME))
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(demo_canvas.created_by(), Actor::system().into());
    assert_eq!(
        find_dashboard_of(&app_state, demo_canvas.id)
            .await
            .modified_by(),
        Actor::system().into()
    );

    let demo_notebook = find_notebook_of(&app_state, demo_canvas.id).await;
    let demo_cells = find_cells_of(&app_state, demo_notebook.id).await;
    assert!(!demo_cells.is_empty());
    for cell in &demo_cells {
        assert_eq!(cell.created_by(), Actor::system().into());
        assert_eq!(cell.modified_by(), Actor::system().into());
    }
}

#[tokio::test]
async fn membership_changes_name_who_made_them() {
    let app_state = init_test_app_state_with_single_sign_on("attribution_membership").await;
    let fixture = create_tenant_with_owner(&app_state, "acme").await;
    let owner = resolve_context(&app_state, fixture.tenant_id, fixture.user_id).await;
    let invitee = insert_user(&app_state, "invitee").await;
    let invitee_email = entity::user::Entity::find_by_id(invitee)
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap()
        .email;

    let invitation = crate::tenant::invitation::service::create_invitation(
        &app_state,
        &owner,
        CreateInvitationDto {
            email: invitee_email,
            role_name: SystemRole::Viewer.as_str().to_string(),
        },
    )
    .await
    .unwrap();
    crate::tenant::invitation::service::accept_invitation(
        &app_state,
        UserId::from_system_unchecked(invitee),
        invitation.id,
    )
    .await
    .unwrap();

    let accepted = find_membership(&app_state, fixture.tenant_id, invitee).await;
    assert_eq!(accepted.created_by(), human_actor(invitee).into());
    assert_eq!(accepted.modified_by(), human_actor(invitee).into());

    crate::tenant::membership::service::update_member_role(
        &app_state,
        &owner,
        invitee,
        SystemRole::Member.as_str(),
    )
    .await
    .unwrap();

    let promoted = find_membership(&app_state, fixture.tenant_id, invitee).await;
    assert_eq!(promoted.created_by(), human_actor(invitee).into());
    assert_eq!(promoted.modified_by(), owner.actor().into());
}

#[tokio::test]
async fn a_new_workspace_names_its_creator_and_leaves_roles_to_the_system() {
    let app_state = init_test_app_state("attribution_new_workspace").await;
    crate::initialize(&app_state).await.unwrap();
    let administrator = find_admin_user_id(&app_state).await;

    let created = crate::tenant::service::create_tenant_for_user(
        &app_state,
        UserId::from_system_unchecked(administrator),
        "Analytics".to_string(),
    )
    .await
    .unwrap();

    assert_eq!(created.created_by(), human_actor(administrator).into());
    for role in find_roles_of(&app_state, created.id).await {
        assert_eq!(
            role.created_by(),
            Actor::system().into(),
            "role {}",
            role.name
        );
    }
    assert_eq!(
        find_membership(&app_state, created.id, administrator)
            .await
            .created_by(),
        human_actor(administrator).into()
    );
}

async fn find_cell(app_state: &AppState, cell_id: Uuid) -> entity::cell::Model {
    entity::cell::Entity::find_by_id(cell_id)
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap()
}

async fn find_conversation(
    app_state: &AppState,
    conversation_id: Uuid,
) -> entity::conversation::Model {
    entity::conversation::Entity::find_by_id(conversation_id)
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap()
}

async fn create_cell_as(
    app_state: &AppState,
    context: &TenantContext,
    notebook_id: Uuid,
    connection_id: Uuid,
    name: &str,
) -> Uuid {
    crate::cell::service::create_cell(
        app_state.clone(),
        context,
        CreateCellDto {
            notebook_id,
            connection_id: Some(connection_id),
            name: Some(name.to_string()),
            content: Some("select 1".to_string()),
            display_order: None,
            cell_type: CellType::Sql,
        },
    )
    .await
    .unwrap()
    .id
}

async fn assert_canvas_last_changed_by(
    app_state: &AppState,
    canvas_id: Uuid,
    actor: Actor,
    step: &str,
) {
    assert_eq!(
        find_canvas(app_state, canvas_id).await.modified_by(),
        actor.into(),
        "after {step}"
    );
}

async fn assert_conversation_last_changed_by(
    app_state: &AppState,
    conversation_id: Uuid,
    actor: Actor,
    step: &str,
) {
    assert_eq!(
        find_conversation(app_state, conversation_id)
            .await
            .modified_by(),
        actor.into(),
        "after {step}"
    );
}

async fn make_ai_live(app_state: &AppState, tenant_id: Uuid) {
    entity::ai_configuration::ActiveModel {
        tenant_id: Set(tenant_id),
        provider: Set(Provider::Anthropic.as_db_str().to_string()),
        api_key_encrypted: Set("unused".to_string()),
        liveness_status: Set(LIVE_AI_STATUS.to_string()),
        last_checked_at: Set(None),
        created_by_user_id: Set(None),
        created_by_actor: Set(Actor::system().kind_name()),
        modified_by_user_id: Set(None),
        modified_by_actor: Set(Actor::system().kind_name()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();
}

async fn edit_first_sql_cell_through_the_agent(
    app_state: &AppState,
    owner: &TenantContext,
    actor: Actor,
    connection_id: Uuid,
    canvas_id: Uuid,
) {
    let canvas = find_canvas(app_state, canvas_id).await;
    crate::canvas::service::apply_canvas_plan(
        app_state,
        owner.tenant_id(),
        CanvasAuthorship {
            ownership: ResourceOwnership {
                owner_user_id: owner.user_id().get_uuid(),
                visibility: Visibility::Tenant,
            },
            actor,
        },
        connection_id,
        CanvasPlan {
            target_canvas_index: None,
            reasoning: String::new(),
            name: canvas.name,
            description: String::new(),
            cells: Vec::new(),
        },
        Some(canvas_id),
        vec![CanvasOp::Update {
            index: 0,
            title: Some("AI title".to_string()),
            sql: Some("select 2".to_string()),
            chart: None,
        }],
    )
    .await
    .unwrap();
}

fn question(text: &str) -> Vec<MessageSegment> {
    vec![MessageSegment::Text {
        text: text.to_string(),
    }]
}

#[tokio::test]
async fn content_edits_stamp_their_canvas() {
    let app_state = init_test_app_state("attribution_canvas_root").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let alice = context_with_role(&app_state, tenant_id, SystemRole::Member, "alice").await;
    let editor = context_with_role(&app_state, tenant_id, SystemRole::Admin, "editor").await;
    let bob = context_with_role(&app_state, tenant_id, SystemRole::Member, "bob").await;
    let connection_id = insert_connection(
        &app_state,
        tenant_id,
        "warehouse",
        alice.user_id().get_uuid(),
        Visibility::Tenant,
    )
    .await;
    let canvas_id = create_canvas_as(&app_state, &alice, "revenue").await;
    let notebook_id = find_notebook_of(&app_state, canvas_id).await.id;
    let first_cell_id =
        create_cell_as(&app_state, &alice, notebook_id, connection_id, "first").await;
    let second_cell_id =
        create_cell_as(&app_state, &alice, notebook_id, connection_id, "second").await;

    edit_first_sql_cell_through_the_agent(
        &app_state,
        &alice,
        bob.ai_actor(),
        connection_id,
        canvas_id,
    )
    .await;
    assert_eq!(
        find_cell(&app_state, first_cell_id).await.modified_by(),
        bob.ai_actor().into()
    );
    assert_canvas_last_changed_by(&app_state, canvas_id, bob.ai_actor(), "an AI edit").await;

    let mut edited_by_alice = CellDto::from(find_cell(&app_state, first_cell_id).await);
    edited_by_alice.content = Some("select 3".to_string());
    crate::cell::service::update_cell(app_state.clone(), &alice, first_cell_id, edited_by_alice)
        .await
        .unwrap();
    assert_eq!(
        find_cell(&app_state, first_cell_id).await.modified_by(),
        alice.actor().into(),
        "a person editing the cell after the AI takes it back"
    );
    assert_eq!(
        find_cell(&app_state, first_cell_id).await.created_by(),
        alice.actor().into()
    );
    assert_canvas_last_changed_by(&app_state, canvas_id, alice.actor(), "a human edit").await;

    crate::cell::service::reorder_cell(
        app_state.clone(),
        &editor,
        second_cell_id,
        ReorderCellDto {
            movement_type: CellDisplayOrderMovement::Up,
        },
    )
    .await
    .unwrap();
    assert_canvas_last_changed_by(&app_state, canvas_id, editor.actor(), "a reorder").await;

    let dashboard_id = find_dashboard_of(&app_state, canvas_id).await.id;
    crate::dashboard::service::update_dashboard(
        app_state.clone(),
        &alice,
        dashboard_id,
        DashboardDto {
            id: dashboard_id,
            name: None,
            description: None,
            layout: None,
            status: DashboardStatus::Active,
        },
    )
    .await
    .unwrap();
    assert_canvas_last_changed_by(&app_state, canvas_id, alice.actor(), "a dashboard edit").await;

    crate::cell::service::delete_cell(app_state.clone(), &editor, second_cell_id)
        .await
        .unwrap();
    assert_canvas_last_changed_by(&app_state, canvas_id, editor.actor(), "a delete").await;

    create_cell_as(&app_state, &alice, notebook_id, connection_id, "third").await;
    assert_canvas_last_changed_by(&app_state, canvas_id, alice.actor(), "a create").await;

    assert_eq!(
        find_canvas(&app_state, canvas_id).await.created_by(),
        alice.actor().into(),
        "content edits never change who created the canvas"
    );
}

#[tokio::test]
async fn messages_stamp_their_conversation() {
    let app_state = init_test_app_state("attribution_conversation_root").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let alice = context_with_role(&app_state, tenant_id, SystemRole::Member, "alice").await;
    make_ai_live(&app_state, tenant_id).await;
    let connection_id = insert_connection(
        &app_state,
        tenant_id,
        "warehouse",
        alice.user_id().get_uuid(),
        Visibility::Tenant,
    )
    .await;

    let conversation_id =
        crate::conversational_analytics::conversation::service::create_conversation(
            app_state.clone(),
            &alice,
            CreateConversationDto {
                connection_id,
                segments: question("How many orders?"),
            },
        )
        .await
        .unwrap()
        .id;
    assert_conversation_last_changed_by(&app_state, conversation_id, alice.actor(), "creation")
        .await;

    let transaction = app_state.database_connection.begin().await.unwrap();
    let answer = crate::conversational_analytics::message::service::create_message(
        &transaction,
        alice.tenant_id(),
        CreateConversationMessageDto {
            conversation_id,
            sequence_number: 2,
            sender: Sender::Assistant.to_string(),
            content: String::new(),
            status: MessageStatus::Processing.to_string(),
        },
        &alice.ai_actor(),
    )
    .await
    .unwrap();
    transaction.commit().await.unwrap();
    assert_conversation_last_changed_by(&app_state, conversation_id, alice.ai_actor(), "an answer")
        .await;

    crate::grant::service::update_visibility(
        app_state.clone(),
        &alice,
        ResourceType::Conversation.as_str().to_string(),
        conversation_id,
        UpdateVisibilityDto {
            visibility: Visibility::Private.as_str().to_string(),
        },
    )
    .await
    .unwrap();
    assert_conversation_last_changed_by(
        &app_state,
        conversation_id,
        alice.actor(),
        "a visibility change",
    )
    .await;

    crate::conversational_analytics::message::service::update_message_status(
        &app_state.database_connection,
        alice.tenant_id(),
        answer.id,
        MessageStatus::Processed.to_string(),
        &alice.ai_actor(),
    )
    .await
    .unwrap();
    assert_conversation_last_changed_by(
        &app_state,
        conversation_id,
        alice.ai_actor(),
        "the answer finishing",
    )
    .await;
    let finished_answer = entity::conversation_message::Entity::find_by_id(answer.id)
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(finished_answer.created_by(), alice.ai_actor().into());
    assert_eq!(finished_answer.modified_by(), alice.ai_actor().into());

    crate::conversational_analytics::conversation::service::append_user_message(
        app_state.clone(),
        &alice,
        conversation_id,
        AppendUserMessageDto {
            segments: question("And last month?"),
        },
    )
    .await
    .unwrap();
    assert_conversation_last_changed_by(
        &app_state,
        conversation_id,
        alice.actor(),
        "a follow-up question",
    )
    .await;

    assert_eq!(
        find_conversation(&app_state, conversation_id)
            .await
            .created_by(),
        alice.actor().into()
    );
}

fn system_attribution() -> AttributionDto {
    AttributionDto {
        actor: ActorKind::System,
        user: None,
    }
}

fn attribution_of(actor: ActorKind, user_id: Uuid, full_name: &str) -> AttributionDto {
    AttributionDto {
        actor,
        user: Some(AttributedUserDto {
            user_id,
            full_name: full_name.to_string(),
        }),
    }
}

async fn insert_connection_created_by(
    app_state: &AppState,
    tenant_id: Uuid,
    name: &str,
    owner_user_id: Uuid,
    actor: Actor,
) {
    entity::connection::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        owner_user_id: Set(owner_user_id),
        visibility: Set(Visibility::Tenant.as_str().to_string()),
        name: Set(name.to_string()),
        description: Set(None),
        username: Set(None),
        password: Set(None),
        host: Set(None),
        port: Set(None),
        database: Set(cryptography_utils::encrypt(
            app_state.config.encryption_key.to_string(),
            "./data/unused.db".to_string(),
        )
        .unwrap()),
        schema: Set(None),
        source_type: Set("SQLITE".to_string()),
        status: Set("ACTIVE".to_string()),
        created_by_user_id: Set(actor.user_id()),
        created_by_actor: Set(actor.kind_name()),
        modified_by_user_id: Set(actor.user_id()),
        modified_by_actor: Set(actor.kind_name()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();
}

async fn create_canvas_through_agent(
    app_state: &AppState,
    owner: &TenantContext,
    actor: Actor,
    name: &str,
) {
    crate::canvas::service::execute_create_canvas(
        app_state,
        owner.tenant_id(),
        ResourceOwnership {
            owner_user_id: owner.user_id().get_uuid(),
            visibility: Visibility::Tenant,
        },
        &actor,
        CreateCanvasDto {
            name: name.to_string(),
            description: None,
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn responses_name_who_made_and_changed_the_row() {
    let app_state = init_test_app_state("attribution_responses").await;
    crate::initialize(&app_state).await.unwrap();
    let default_tenant_id = entity::tenant::Entity::find()
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap()
        .id;
    let admin_user_id = find_admin_user_id(&app_state).await;
    let admin = resolve_context(&app_state, default_tenant_id, admin_user_id).await;
    let admin_attribution = attribution_of(ActorKind::Human, admin_user_id, TEST_ADMIN_FULL_NAME);

    let canvases = crate::canvas::service::get_all_canvases(app_state.clone(), &admin, 0, 50, None)
        .await
        .unwrap()
        .data;
    let demo_canvas = canvases
        .iter()
        .find(|canvas| canvas.name == DEMO_CANVAS_NAME)
        .unwrap();
    assert_eq!(demo_canvas.created_by, Some(system_attribution()));
    assert_eq!(demo_canvas.modified_by, Some(system_attribution()));

    let renamed = crate::canvas::service::update_canvas(
        app_state.clone(),
        &admin,
        demo_canvas.id,
        CreateCanvasDto {
            name: "Renamed".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(renamed.created_by, Some(system_attribution()));
    assert_eq!(renamed.modified_by, Some(admin_attribution.clone()));
    let fetched = crate::canvas::service::get_canvas(app_state.clone(), &admin, demo_canvas.id)
        .await
        .unwrap();
    assert_eq!(fetched.modified_by, Some(admin_attribution.clone()));

    let created = crate::canvas::service::create_canvas(
        app_state.clone(),
        &admin,
        admin_user_id,
        CreateCanvasDto {
            name: "Mine".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(created.created_by, Some(admin_attribution.clone()));
    assert_eq!(created.modified_by, Some(admin_attribution.clone()));

    let demo_notebook = find_notebook_of(&app_state, demo_canvas.id).await;
    let notebook =
        crate::notebook::service::get_notebook(app_state.clone(), &admin, demo_notebook.id)
            .await
            .unwrap();
    assert!(!notebook.cells.is_empty());
    assert!(
        notebook
            .cells
            .iter()
            .all(|cell| cell.created_by == Some(system_attribution()))
    );
    let mut edited_cell = CellDto::from(find_cell(&app_state, notebook.cells[0].id).await);
    edited_cell.name = Some("Edited".to_string());
    let edited_cell =
        crate::cell::service::update_cell(app_state.clone(), &admin, edited_cell.id, edited_cell)
            .await
            .unwrap();
    assert_eq!(edited_cell.created_by, Some(system_attribution()));
    assert_eq!(edited_cell.modified_by, Some(admin_attribution.clone()));

    let connections =
        crate::connection::service::get_all_connections(app_state.clone(), &admin, None)
            .await
            .unwrap();
    let demo_connection = connections
        .iter()
        .find(|connection| connection.name == DEMO_CONNECTION_NAME)
        .unwrap();
    assert_eq!(demo_connection.created_by, Some(system_attribution()));
    let fetched_connection =
        crate::connection::service::get_connection(app_state.clone(), &admin, demo_connection.id)
            .await
            .unwrap();
    assert_eq!(fetched_connection.modified_by, Some(system_attribution()));

    make_ai_live(&app_state, default_tenant_id).await;
    let conversation = crate::conversational_analytics::conversation::service::create_conversation(
        app_state.clone(),
        &admin,
        CreateConversationDto {
            connection_id: demo_connection.id,
            segments: question("How many orders?"),
        },
    )
    .await
    .unwrap();
    assert_eq!(conversation.created_by, Some(admin_attribution.clone()));
    let listed = crate::conversational_analytics::conversation::service::list_conversations(
        app_state.clone(),
        &admin,
        ListConversationsQuery {
            page: None,
            page_size: None,
            search: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(
        listed.items[0].conversation.modified_by,
        Some(admin_attribution)
    );
}

#[tokio::test]
async fn created_by_me_includes_work_the_ai_did_for_me() {
    let app_state = init_test_app_state("attribution_created_by_me").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let alice = context_with_role(&app_state, tenant_id, SystemRole::Member, "alice").await;
    let bob = context_with_role(&app_state, tenant_id, SystemRole::Member, "bob").await;

    create_canvas_as(&app_state, &alice, "alice by hand").await;
    create_canvas_through_agent(&app_state, &alice, alice.ai_actor(), "ai for alice").await;
    create_canvas_through_agent(&app_state, &alice, bob.ai_actor(), "ai for bob in alice's").await;
    create_canvas_as(&app_state, &bob, "bob by hand").await;

    let all_canvases =
        crate::canvas::service::get_all_canvases(app_state.clone(), &alice, 0, 50, None)
            .await
            .unwrap();
    assert_eq!(all_canvases.data.len(), 4);

    let mine = crate::canvas::service::get_all_canvases(
        app_state.clone(),
        &alice,
        0,
        50,
        Some(CreatedByFilter::Me),
    )
    .await
    .unwrap();
    let mut my_canvas_names: Vec<String> =
        mine.data.into_iter().map(|canvas| canvas.name).collect();
    my_canvas_names.sort();
    assert_eq!(my_canvas_names, vec!["ai for alice", "alice by hand"]);
    assert_eq!(mine.total_items, 2);

    let alice_user_id = alice.user_id().get_uuid();
    insert_connection_created_by(
        &app_state,
        tenant_id,
        "alice by hand",
        alice_user_id,
        alice.actor(),
    )
    .await;
    insert_connection_created_by(
        &app_state,
        tenant_id,
        "ai for alice",
        alice_user_id,
        alice.ai_actor(),
    )
    .await;
    insert_connection_created_by(
        &app_state,
        tenant_id,
        "ai for bob",
        alice_user_id,
        bob.ai_actor(),
    )
    .await;
    insert_connection_created_by(
        &app_state,
        tenant_id,
        "startup",
        alice_user_id,
        Actor::system(),
    )
    .await;

    let my_connections = crate::connection::service::get_all_connections(
        app_state.clone(),
        &alice,
        Some(CreatedByFilter::Me),
    )
    .await
    .unwrap();
    let mut my_connection_names: Vec<String> = my_connections
        .into_iter()
        .map(|connection| connection.name)
        .collect();
    my_connection_names.sort();
    assert_eq!(my_connection_names, vec!["ai for alice", "alice by hand"]);
}

#[tokio::test]
async fn an_unknown_created_by_filter_is_rejected() {
    let parse = |uri: &str| {
        axum::extract::Query::<CreatedByQuery>::try_from_uri(&uri.parse().unwrap())
            .unwrap()
            .filter()
    };

    assert_eq!(
        parse("/canvas?created_by=me").unwrap(),
        Some(CreatedByFilter::Me)
    );
    assert_eq!(parse("/canvas?page=1").unwrap(), None);
    assert!(parse("/canvas?created_by=everyone").is_err());
    assert!(parse("/canvas?created_by=").is_err());
}

const WRITE_ORDER_TRIGGERS: &str = "
    CREATE TABLE write_log (sequence INTEGER PRIMARY KEY AUTOINCREMENT, table_name TEXT NOT NULL);
    CREATE TRIGGER log_canvas_update AFTER UPDATE ON canvas BEGIN INSERT INTO write_log (table_name) VALUES ('canvas'); END;
    CREATE TRIGGER log_canvas_delete AFTER DELETE ON canvas BEGIN INSERT INTO write_log (table_name) VALUES ('canvas'); END;
    CREATE TRIGGER log_cell_insert AFTER INSERT ON cell BEGIN INSERT INTO write_log (table_name) VALUES ('cell'); END;
    CREATE TRIGGER log_cell_update AFTER UPDATE ON cell BEGIN INSERT INTO write_log (table_name) VALUES ('cell'); END;
    CREATE TRIGGER log_cell_delete AFTER DELETE ON cell BEGIN INSERT INTO write_log (table_name) VALUES ('cell'); END;
    CREATE TRIGGER log_notebook_delete AFTER DELETE ON notebook BEGIN INSERT INTO write_log (table_name) VALUES ('notebook'); END;
    CREATE TRIGGER log_dashboard_update AFTER UPDATE ON dashboard BEGIN INSERT INTO write_log (table_name) VALUES ('dashboard'); END;
    CREATE TRIGGER log_dashboard_delete AFTER DELETE ON dashboard BEGIN INSERT INTO write_log (table_name) VALUES ('dashboard'); END;
";

async fn take_write_order(app_state: &AppState) -> Vec<String> {
    let rows = app_state
        .database_connection
        .query_all(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT table_name FROM write_log ORDER BY sequence",
        ))
        .await
        .unwrap();
    app_state
        .database_connection
        .execute_unprepared("DELETE FROM write_log")
        .await
        .unwrap();
    rows.iter()
        .map(|row| row.try_get::<String>("", "table_name").unwrap())
        .collect()
}

async fn assert_canvas_written_first(app_state: &AppState, step: &str) {
    let write_order = take_write_order(app_state).await;
    assert!(
        write_order.len() > 1 && write_order[0] == "canvas",
        "{step} must write its canvas before any content, wrote {write_order:?}"
    );
}

#[tokio::test]
async fn content_writes_take_the_canvas_first() {
    let app_state = init_test_app_state("attribution_canvas_first").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let alice = context_with_role(&app_state, tenant_id, SystemRole::Member, "alice").await;
    let editor = context_with_role(&app_state, tenant_id, SystemRole::Admin, "editor").await;
    let bob = context_with_role(&app_state, tenant_id, SystemRole::Member, "bob").await;
    let connection_id = insert_connection(
        &app_state,
        tenant_id,
        "warehouse",
        alice.user_id().get_uuid(),
        Visibility::Tenant,
    )
    .await;
    let canvas_id = create_canvas_as(&app_state, &alice, "revenue").await;
    let notebook_id = find_notebook_of(&app_state, canvas_id).await.id;
    let dashboard_id = find_dashboard_of(&app_state, canvas_id).await.id;
    let first_cell_id =
        create_cell_as(&app_state, &alice, notebook_id, connection_id, "first").await;
    create_cell_as(&app_state, &alice, notebook_id, connection_id, "second").await;
    app_state
        .database_connection
        .execute_unprepared(WRITE_ORDER_TRIGGERS)
        .await
        .unwrap();

    let third_cell_id =
        create_cell_as(&app_state, &editor, notebook_id, connection_id, "third").await;
    assert_canvas_written_first(&app_state, "a cell create").await;

    let mut edited = CellDto::from(find_cell(&app_state, first_cell_id).await);
    edited.content = Some("select 2".to_string());
    crate::cell::service::update_cell(app_state.clone(), &editor, first_cell_id, edited)
        .await
        .unwrap();
    assert_canvas_written_first(&app_state, "a cell edit").await;

    crate::cell::service::reorder_cell(
        app_state.clone(),
        &editor,
        third_cell_id,
        ReorderCellDto {
            movement_type: CellDisplayOrderMovement::Top,
        },
    )
    .await
    .unwrap();
    assert_canvas_written_first(&app_state, "a reorder").await;

    crate::dashboard::service::update_dashboard(
        app_state.clone(),
        &editor,
        dashboard_id,
        DashboardDto {
            id: dashboard_id,
            name: None,
            description: None,
            layout: None,
            status: DashboardStatus::Active,
        },
    )
    .await
    .unwrap();
    assert_canvas_written_first(&app_state, "a dashboard edit").await;

    edit_first_sql_cell_through_the_agent(
        &app_state,
        &alice,
        bob.ai_actor(),
        connection_id,
        canvas_id,
    )
    .await;
    assert_canvas_written_first(&app_state, "an AI plan on an existing canvas").await;

    crate::cell::service::delete_cell(app_state.clone(), &alice, third_cell_id)
        .await
        .unwrap();
    assert_canvas_written_first(&app_state, "a cell delete").await;

    crate::canvas::service::delete_canvas(app_state.clone(), &alice, canvas_id)
        .await
        .unwrap();
    assert_canvas_written_first(&app_state, "a canvas delete").await;
}

#[tokio::test]
async fn reordering_cells_keeps_who_last_edited_each_cell() {
    let app_state = init_test_app_state("attribution_reorder").await;
    let tenant_id = create_tenant(&app_state, "t").await;
    let alice = context_with_role(&app_state, tenant_id, SystemRole::Member, "alice").await;
    let editor = context_with_role(&app_state, tenant_id, SystemRole::Admin, "editor").await;
    let bob = context_with_role(&app_state, tenant_id, SystemRole::Member, "bob").await;
    let connection_id = insert_connection(
        &app_state,
        tenant_id,
        "warehouse",
        alice.user_id().get_uuid(),
        Visibility::Tenant,
    )
    .await;
    let canvas_id = create_canvas_as(&app_state, &alice, "revenue").await;
    let notebook_id = find_notebook_of(&app_state, canvas_id).await.id;
    let first_cell_id =
        create_cell_as(&app_state, &alice, notebook_id, connection_id, "first").await;
    let second_cell_id =
        create_cell_as(&app_state, &alice, notebook_id, connection_id, "second").await;
    let third_cell_id =
        create_cell_as(&app_state, &alice, notebook_id, connection_id, "third").await;

    let transaction = app_state.database_connection.begin().await.unwrap();
    crate::cell::service::update_cell_content_transaction(
        &transaction,
        alice.tenant_id(),
        first_cell_id,
        None,
        Some("select 2".to_string()),
        &bob.ai_actor(),
    )
    .await
    .unwrap();
    transaction.commit().await.unwrap();

    crate::cell::service::reorder_cell(
        app_state.clone(),
        &editor,
        third_cell_id,
        ReorderCellDto {
            movement_type: CellDisplayOrderMovement::Top,
        },
    )
    .await
    .unwrap();

    let first_cell = find_cell(&app_state, first_cell_id).await;
    let second_cell = find_cell(&app_state, second_cell_id).await;
    let third_cell = find_cell(&app_state, third_cell_id).await;
    assert_eq!(
        [
            first_cell.display_order,
            second_cell.display_order,
            third_cell.display_order
        ],
        [2, 3, 1]
    );
    assert_eq!(
        first_cell.modified_by(),
        bob.ai_actor().into(),
        "moving another cell keeps the AI's edit on this one"
    );
    assert_eq!(second_cell.modified_by(), alice.actor().into());
    assert_eq!(
        third_cell.modified_by(),
        alice.actor().into(),
        "moving a cell is not an edit of the cell"
    );
    assert_eq!(
        find_canvas(&app_state, canvas_id).await.modified_by(),
        editor.actor().into(),
        "a reorder is a change to the canvas"
    );
}
