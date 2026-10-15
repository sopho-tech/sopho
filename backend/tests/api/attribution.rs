use crate::support::canvas_builder::{create_canvas, create_sql_cell, string_field};
use crate::support::constants::{
    ACTIVE_STATUS, ADMIN_EMAIL, ADMIN_FULL_NAME, ADMIN_ROLE, AI_ACTOR, ASSISTANT_SENDER,
    ATTRIBUTED_TABLES, ATTRIBUTION_COLUMNS, CANVAS_RESOURCE, CONVERSATION_RESOURCE,
    DEMO_CANVAS_NAME, DEMO_CONNECTION_NAME, EDITOR_ACCESS, HUMAN_ACTOR, LIVE_AI_PROVIDER,
    MEMBER_ROLE, ORDER_STATUS_QUESTION, OWNER_ROLE, PLATFORM_MEMBER, POSTGRES_WRITE_ORDER_TRIGGERS,
    PRIVATE_VISIBILITY, SQLITE_WRITE_ORDER_TRIGGERS, SYSTEM_ACTOR, SYSTEM_ROLES, TENANT_VISIBILITY,
    UNKNOWN_CREATED_BY_FILTER_MESSAGE, VIEWER_ROLE,
};
use crate::support::conversation::{
    answer_order_status_question, append_canvas_command, event_data, event_names, run_completion,
    start_conversation, wait_for_persisted_event,
};
use crate::support::database::{
    Column, backend, count_rows, now, select_rows, select_uuid, try_insert_row, update_rows,
};
use crate::support::records::{insert_grant, set_owner_and_visibility};
use crate::support::workspaces::{
    WorkspaceMember, create_user, create_workspace, join_workspace, sign_in,
};
use crate::support::{TestApp, UserSession};
use reqwest::StatusCode;
use sea_orm::{ConnectionTrait, DbBackend, DbErr, QueryResult, Statement};
use serde_json::{Value, json};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
struct RecordedActor {
    kind: String,
    user_id: Option<Uuid>,
}

impl RecordedActor {
    fn system() -> Self {
        Self {
            kind: SYSTEM_ACTOR.to_string(),
            user_id: None,
        }
    }

    fn person(user_id: Uuid) -> Self {
        Self {
            kind: HUMAN_ACTOR.to_string(),
            user_id: Some(user_id),
        }
    }

    fn ai_for(user_id: Uuid) -> Self {
        Self {
            kind: AI_ACTOR.to_string(),
            user_id: Some(user_id),
        }
    }
}

struct RecordedAttribution {
    created_by: RecordedActor,
    modified_by: RecordedActor,
}

struct Team {
    workspace: Uuid,
    editor: WorkspaceMember,
    alice: WorkspaceMember,
    bob: WorkspaceMember,
    connection_id: Uuid,
}

struct ConversationCanvas {
    conversation_id: String,
    canvas_id: Uuid,
    notebook_id: Uuid,
    sql_cell_id: Uuid,
    chart_cell_id: Uuid,
    dashboard: Value,
}

fn uuid_field(value: &Value, field: &str) -> Uuid {
    parse_uuid(&string_field(value, field))
}

fn parse_uuid(text: &str) -> Uuid {
    text.parse()
        .unwrap_or_else(|error| panic!("{text} is not a UUID: {error}"))
}

fn recorded_actor(row: &QueryResult, actor_column: &str, user_column: &str) -> RecordedActor {
    RecordedActor {
        kind: row
            .try_get("", actor_column)
            .unwrap_or_else(|error| panic!("read {actor_column}: {error}")),
        user_id: row
            .try_get("", user_column)
            .unwrap_or_else(|error| panic!("read {user_column}: {error}")),
    }
}

async fn attributions(
    app: &TestApp,
    table: &str,
    filters: &[Column<'_>],
) -> Vec<RecordedAttribution> {
    select_rows(app, table, &ATTRIBUTION_COLUMNS, filters)
        .await
        .iter()
        .map(|row| RecordedAttribution {
            created_by: recorded_actor(row, "created_by_actor", "created_by_user_id"),
            modified_by: recorded_actor(row, "modified_by_actor", "modified_by_user_id"),
        })
        .collect()
}

async fn attribution(app: &TestApp, table: &str, filters: &[Column<'_>]) -> RecordedAttribution {
    let mut rows = attributions(app, table, filters).await;
    assert_eq!(rows.len(), 1, "expected one {table} row");
    rows.remove(0)
}

async fn attribution_of(app: &TestApp, table: &str, id: Uuid) -> RecordedAttribution {
    attribution(app, table, &[("id", id.into())]).await
}

async fn record_as_made_by(app: &TestApp, table: &str, id: Uuid, actor: &RecordedActor) {
    let updated = update_rows(
        app,
        table,
        &[
            ("created_by_actor", actor.kind.as_str().into()),
            ("created_by_user_id", actor.user_id.into()),
            ("modified_by_actor", actor.kind.as_str().into()),
            ("modified_by_user_id", actor.user_id.into()),
        ],
        &[("id", id.into())],
    )
    .await;
    assert_eq!(updated, 1, "expected one {table} row to update");
}

async fn record_as_last_changed_by(app: &TestApp, table: &str, id: Uuid, actor: &RecordedActor) {
    let updated = update_rows(
        app,
        table,
        &[
            ("modified_by_actor", actor.kind.as_str().into()),
            ("modified_by_user_id", actor.user_id.into()),
        ],
        &[("id", id.into())],
    )
    .await;
    assert_eq!(updated, 1, "expected one {table} row to update");
}

async fn admin_user_id(app: &TestApp) -> Uuid {
    select_uuid(app, "user", "id", &[("email", ADMIN_EMAIL.into())]).await
}

async fn form_team(app: &TestApp) -> Team {
    let workspace = create_workspace(app, "t").await;
    let editor = join_workspace(app, workspace, "editor", ADMIN_ROLE).await;
    let alice = join_workspace(app, workspace, "alice", MEMBER_ROLE).await;
    let bob = join_workspace(app, workspace, "bob", MEMBER_ROLE).await;
    let connection_id = uuid_field(
        &app.create_named_sqlite_demo_connection_as(&editor.session, "warehouse")
            .await,
        "id",
    );
    set_owner_and_visibility(
        app,
        "connection",
        connection_id,
        alice.user.id,
        TENANT_VISIBILITY,
    )
    .await;
    Team {
        workspace,
        editor,
        alice,
        bob,
        connection_id,
    }
}

async fn run_canvas_command(session: &UserSession, conversation_id: &str) -> Value {
    append_canvas_command(session, conversation_id).await;
    let events = run_completion(session, conversation_id).await;
    let generated = event_data(&events, "canvas_generated");
    wait_for_persisted_event(session, conversation_id, "completed").await;
    generated
}

async fn canvas_from_conversation(
    session: &UserSession,
    connection_id: Uuid,
) -> ConversationCanvas {
    let conversation_id = answer_order_status_question(session, &connection_id.to_string()).await;
    wait_for_persisted_event(session, &conversation_id, "completed").await;
    let canvas_id = uuid_field(
        &run_canvas_command(session, &conversation_id).await,
        "canvas_id",
    );
    let notebooks = session.get(&format!("/notebook/canvas/{canvas_id}")).await;
    assert_eq!(notebooks.status, StatusCode::OK, "{}", notebooks.body);
    let notebook_id = uuid_field(&notebooks.body[0], "id");
    let cells = session.get(&format!("/notebook/{notebook_id}/cells")).await;
    assert_eq!(cells.status, StatusCode::OK, "{}", cells.body);
    let cell_of_type = |cell_type: &str| {
        cells.body["cells"]
            .as_array()
            .expect("notebook cells")
            .iter()
            .find(|cell| cell["cell_type"] == cell_type)
            .map(|cell| uuid_field(cell, "id"))
            .unwrap_or_else(|| panic!("the generated canvas has no {cell_type} cell"))
    };
    let sql_cell_id = cell_of_type("SQL");
    let chart_cell_id = cell_of_type("CHART");
    let dashboard = session.get(&format!("/dashboard/canvas/{canvas_id}")).await;
    assert_eq!(dashboard.status, StatusCode::OK, "{}", dashboard.body);
    ConversationCanvas {
        conversation_id,
        canvas_id,
        notebook_id,
        sql_cell_id,
        chart_cell_id,
        dashboard: dashboard.body,
    }
}

async fn let_bob_edit_the_conversation(app: &TestApp, team: &Team, conversation_id: &str) {
    insert_grant(
        app,
        team.workspace,
        CONVERSATION_RESOURCE,
        parse_uuid(conversation_id),
        team.bob.user.id,
        EDITOR_ACCESS,
    )
    .await;
}

async fn extend_canvas_through_the_agent(session: &UserSession, canvas: &ConversationCanvas) {
    let extended = run_canvas_command(session, &canvas.conversation_id).await;
    assert_eq!(extended["reused"], true, "{extended}");
    assert_eq!(uuid_field(&extended, "canvas_id"), canvas.canvas_id);
}

async fn edit_cell_content(session: &UserSession, cell_id: Uuid, content: &str) {
    let cell = session.get(&format!("/cell/{cell_id}")).await;
    assert_eq!(cell.status, StatusCode::OK, "{}", cell.body);
    let mut edited = cell.body;
    edited["content"] = json!(content);
    let updated = session.put(&format!("/cell/{cell_id}"), edited).await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.body);
}

async fn reorder_cell(session: &UserSession, cell_id: Uuid, movement_type: &str) {
    let reordered = session
        .patch(
            &format!("/cell/{cell_id}/reorder"),
            json!({ "movement_type": movement_type }),
        )
        .await;
    assert_eq!(reordered.status, StatusCode::OK, "{}", reordered.body);
}

async fn update_dashboard(session: &UserSession, dashboard: &Value) {
    let updated = session
        .put(
            &format!("/dashboard/{}", string_field(dashboard, "id")),
            dashboard.clone(),
        )
        .await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.body);
}

async fn delete_cell(session: &UserSession, cell_id: Uuid) {
    let deleted = session.delete(&format!("/cell/{cell_id}")).await;
    assert!(deleted.status.is_success(), "{}", deleted.body);
}

async fn change_visibility(
    session: &UserSession,
    resource_type: &str,
    resource_id: Uuid,
    visibility: &str,
) {
    let changed = session
        .put(
            &format!("/grant/{resource_type}/{resource_id}/visibility"),
            json!({ "visibility": visibility }),
        )
        .await;
    assert_eq!(changed.status, StatusCode::OK, "{}", changed.body);
}

async fn assert_canvas_last_changed_by(
    app: &TestApp,
    canvas_id: Uuid,
    actor: &RecordedActor,
    step: &str,
) {
    assert_eq!(
        &attribution_of(app, "canvas", canvas_id).await.modified_by,
        actor,
        "after {step}"
    );
}

async fn assert_conversation_last_changed_by(
    app: &TestApp,
    conversation_id: Uuid,
    actor: &RecordedActor,
    step: &str,
) {
    assert_eq!(
        &attribution_of(app, "conversation", conversation_id)
            .await
            .modified_by,
        actor,
        "after {step}"
    );
}

async fn column_nullability(app: &TestApp, table: &str) -> HashMap<String, bool> {
    let database_backend = backend(app);
    let query = match database_backend {
        DbBackend::Sqlite => format!(
            "SELECT name AS column_name, \"notnull\" = 0 AS is_nullable FROM pragma_table_info('{table}')"
        ),
        DbBackend::Postgres => format!(
            "SELECT column_name::text AS column_name, is_nullable = 'YES' AS is_nullable \
             FROM information_schema.columns \
             WHERE table_schema = current_schema() AND table_name = '{table}'"
        ),
        unsupported => panic!("{unsupported:?} is not a metadata database"),
    };
    app.app_state
        .database_connection
        .query_all(Statement::from_string(database_backend, query))
        .await
        .unwrap_or_else(|error| panic!("read the columns of {table}: {error}"))
        .iter()
        .map(|column| {
            (
                column
                    .try_get::<String>("", "column_name")
                    .expect("read the column name"),
                column
                    .try_get::<bool>("", "is_nullable")
                    .expect("read the column nullability"),
            )
        })
        .collect()
}

async fn insert_attributed_canvas(
    app: &TestApp,
    workspace: Uuid,
    owner_user_id: Uuid,
    created_by: &RecordedActor,
    modified_by: &RecordedActor,
) -> Result<(), DbErr> {
    try_insert_row(
        app,
        "canvas",
        &[
            ("id", Uuid::new_v4().into()),
            ("tenant_id", workspace.into()),
            ("owner_user_id", owner_user_id.into()),
            ("visibility", TENANT_VISIBILITY.into()),
            ("name", "checked".into()),
            ("description", sea_orm::Value::String(None)),
            ("status", ACTIVE_STATUS.into()),
            ("created_by_user_id", created_by.user_id.into()),
            ("created_by_actor", created_by.kind.as_str().into()),
            ("modified_by_user_id", modified_by.user_id.into()),
            ("modified_by_actor", modified_by.kind.as_str().into()),
            ("created_at", now().into()),
            ("updated_at", now().into()),
        ],
    )
    .await
}

fn assert_refused_by_a_check(inserted: Result<(), DbErr>, description: &str) {
    let error = inserted.expect_err(&format!("{description} must be refused"));
    assert!(
        error
            .to_string()
            .to_lowercase()
            .contains("check constraint"),
        "{description} must fail a check constraint: {error}"
    );
}

async fn record_write_order(app: &TestApp) {
    let triggers = match backend(app) {
        DbBackend::Sqlite => SQLITE_WRITE_ORDER_TRIGGERS,
        DbBackend::Postgres => POSTGRES_WRITE_ORDER_TRIGGERS,
        unsupported => panic!("{unsupported:?} is not a metadata database"),
    };
    app.app_state
        .database_connection
        .execute_unprepared(triggers)
        .await
        .expect("install the write-order triggers");
}

async fn take_write_order(app: &TestApp) -> Vec<String> {
    let rows = app
        .app_state
        .database_connection
        .query_all(Statement::from_string(
            backend(app),
            "SELECT table_name FROM write_log ORDER BY write_sequence",
        ))
        .await
        .expect("read the write log");
    app.app_state
        .database_connection
        .execute_unprepared("DELETE FROM write_log")
        .await
        .expect("clear the write log");
    rows.iter()
        .map(|row| {
            row.try_get::<String>("", "table_name")
                .expect("read the written table")
        })
        .collect()
}

async fn assert_canvas_written_first(app: &TestApp, step: &str) {
    let write_order = take_write_order(app).await;
    assert!(
        write_order.len() > 1 && write_order[0] == "canvas",
        "{step} must write its canvas before any content, wrote {write_order:?}"
    );
}

async fn display_order(app: &TestApp, cell_id: Uuid) -> i32 {
    let rows = select_rows(app, "cell", &["display_order"], &[("id", cell_id.into())]).await;
    assert_eq!(rows.len(), 1, "expected one cell row");
    rows[0]
        .try_get("", "display_order")
        .expect("read the display order")
}

fn system_attribution() -> Value {
    json!({ "actor": SYSTEM_ACTOR, "user": null })
}

fn person_attribution(user_id: Uuid, full_name: &str) -> Value {
    json!({
        "actor": HUMAN_ACTOR,
        "user": { "user_id": user_id, "full_name": full_name },
    })
}

fn sorted_names(items: &Value) -> Vec<String> {
    let mut names: Vec<String> = items
        .as_array()
        .expect("a list")
        .iter()
        .map(|item| string_field(item, "name"))
        .collect();
    names.sort();
    names
}

fn named<'a>(items: &'a Value, name: &str) -> &'a Value {
    items
        .as_array()
        .expect("a list")
        .iter()
        .find(|item| item["name"] == name)
        .unwrap_or_else(|| panic!("{name} is missing from {items}"))
}

#[tokio::test]
async fn every_table_in_scope_carries_attribution_columns() {
    let app = TestApp::spawn().await;

    for table in ATTRIBUTED_TABLES {
        let nullability = column_nullability(&app, table).await;
        for (column, expected_nullable) in [
            ("created_by_user_id", true),
            ("created_by_actor", false),
            ("modified_by_user_id", true),
            ("modified_by_actor", false),
        ] {
            assert_eq!(
                nullability.get(column),
                Some(&expected_nullable),
                "{table}.{column} must exist with nullable = {expected_nullable}"
            );
        }
    }
}

#[tokio::test]
async fn the_database_refuses_unattributed_rows() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let person = create_user(&app, "person", PLATFORM_MEMBER).await.id;
    let valid = RecordedActor::person(person);
    let invalid_pairs = [
        RecordedActor {
            kind: HUMAN_ACTOR.to_string(),
            user_id: None,
        },
        RecordedActor {
            kind: AI_ACTOR.to_string(),
            user_id: None,
        },
        RecordedActor {
            kind: SYSTEM_ACTOR.to_string(),
            user_id: Some(person),
        },
        RecordedActor {
            kind: "ROBOT".to_string(),
            user_id: Some(person),
        },
    ];

    for invalid in &invalid_pairs {
        assert_refused_by_a_check(
            insert_attributed_canvas(&app, workspace, person, invalid, &valid).await,
            &format!("created pair {invalid:?}"),
        );
        assert_refused_by_a_check(
            insert_attributed_canvas(&app, workspace, person, &valid, invalid).await,
            &format!("modified pair {invalid:?}"),
        );
    }

    insert_attributed_canvas(
        &app,
        workspace,
        person,
        &RecordedActor::system(),
        &RecordedActor::system(),
    )
    .await
    .expect("a system row is attributed");
    insert_attributed_canvas(
        &app,
        workspace,
        person,
        &RecordedActor::ai_for(person),
        &valid,
    )
    .await
    .expect("an AI row names the person it worked for");
}

#[tokio::test]
async fn requests_stamp_the_person_who_made_them() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let creator = join_workspace(&app, workspace, "creator", MEMBER_ROLE).await;
    let editor = join_workspace(&app, workspace, "editor", ADMIN_ROLE).await;
    let creator_actor = RecordedActor::person(creator.user.id);
    let editor_actor = RecordedActor::person(editor.user.id);

    let own_canvas = create_canvas(&editor.session, "own").await;
    let others_canvas = create_canvas(&creator.session, "theirs").await;
    let own_canvas_id = parse_uuid(&own_canvas.canvas_id);
    let others_canvas_id = parse_uuid(&others_canvas.canvas_id);

    assert_eq!(
        attribution_of(&app, "notebook", parse_uuid(&others_canvas.notebook_id))
            .await
            .created_by,
        creator_actor
    );
    assert_eq!(
        attribution_of(
            &app,
            "dashboard",
            uuid_field(&others_canvas.dashboard, "id")
        )
        .await
        .created_by,
        creator_actor
    );

    for canvas_id in [own_canvas_id, others_canvas_id] {
        let renamed = editor
            .session
            .put(
                &format!("/canvas/{canvas_id}"),
                json!({ "name": "renamed", "description": null }),
            )
            .await;
        assert_eq!(renamed.status, StatusCode::OK, "{}", renamed.body);
    }

    let own = attribution_of(&app, "canvas", own_canvas_id).await;
    assert_eq!(own.created_by, editor_actor);
    assert_eq!(own.modified_by, editor_actor);

    let others = attribution_of(&app, "canvas", others_canvas_id).await;
    assert_eq!(others.created_by, creator_actor);
    assert_eq!(others.modified_by, editor_actor);

    change_visibility(
        &creator.session,
        CANVAS_RESOURCE,
        others_canvas_id,
        PRIVATE_VISIBILITY,
    )
    .await;
    let shared = attribution_of(&app, "canvas", others_canvas_id).await;
    assert_eq!(shared.created_by, creator_actor);
    assert_eq!(shared.modified_by, creator_actor);
}

#[tokio::test]
async fn agent_work_is_stamped_for_the_person_who_prompted_it() {
    let app = TestApp::spawn().await;
    let team = form_team(&app).await;
    team.editor
        .session
        .configure_live_ai_provider(LIVE_AI_PROVIDER)
        .await;
    let conversation_id =
        answer_order_status_question(&team.alice.session, &team.connection_id.to_string()).await;
    wait_for_persisted_event(&team.alice.session, &conversation_id, "completed").await;
    let_bob_edit_the_conversation(&app, &team, &conversation_id).await;
    let bob_ai = RecordedActor::ai_for(team.bob.user.id);

    let canvas_id = uuid_field(
        &run_canvas_command(&team.bob.session, &conversation_id).await,
        "canvas_id",
    );

    assert_eq!(
        select_uuid(&app, "canvas", "owner_user_id", &[("id", canvas_id.into())]).await,
        team.alice.user.id
    );
    let canvas = attribution_of(&app, "canvas", canvas_id).await;
    assert_eq!(canvas.created_by, bob_ai);
    assert_eq!(canvas.modified_by, bob_ai);

    let notebook_id = select_uuid(&app, "notebook", "id", &[("canvas_id", canvas_id.into())]).await;
    assert_eq!(
        attribution_of(&app, "notebook", notebook_id)
            .await
            .created_by,
        bob_ai
    );
    assert_eq!(
        attribution(&app, "dashboard", &[("canvas_id", canvas_id.into())])
            .await
            .created_by,
        bob_ai
    );

    let cells = attributions(&app, "cell", &[("notebook_id", notebook_id.into())]).await;
    assert!(!cells.is_empty());
    for cell in &cells {
        assert_eq!(cell.created_by, bob_ai);
        assert_eq!(cell.modified_by, bob_ai);
    }
}

#[tokio::test]
async fn startup_work_is_stamped_as_the_system() {
    let app = TestApp::spawn().await;
    let system = RecordedActor::system();

    assert_eq!(count_rows(&app, "tenant", &[]).await, 1);
    let default_tenant = attribution_of(&app, "tenant", app.tenant_id).await;
    assert_eq!(default_tenant.created_by, system);
    assert_eq!(default_tenant.modified_by, system);

    let roles = attributions(&app, "role", &[("tenant_id", app.tenant_id.into())]).await;
    assert_eq!(roles.len(), SYSTEM_ROLES.len());
    for role in &roles {
        assert_eq!(role.created_by, system);
    }

    let admin_user_id = admin_user_id(&app).await;
    assert_eq!(
        attribution(
            &app,
            "tenant_membership",
            &[
                ("tenant_id", app.tenant_id.into()),
                ("user_id", admin_user_id.into()),
            ],
        )
        .await
        .created_by,
        system
    );

    assert_eq!(
        attribution(&app, "connection", &[("name", DEMO_CONNECTION_NAME.into())])
            .await
            .created_by,
        system
    );

    let demo_canvas_id =
        select_uuid(&app, "canvas", "id", &[("name", DEMO_CANVAS_NAME.into())]).await;
    assert_eq!(
        attribution_of(&app, "canvas", demo_canvas_id)
            .await
            .created_by,
        system
    );
    assert_eq!(
        attribution(&app, "dashboard", &[("canvas_id", demo_canvas_id.into())])
            .await
            .modified_by,
        system
    );

    let demo_notebook_id = select_uuid(
        &app,
        "notebook",
        "id",
        &[("canvas_id", demo_canvas_id.into())],
    )
    .await;
    let demo_cells = attributions(&app, "cell", &[("notebook_id", demo_notebook_id.into())]).await;
    assert!(!demo_cells.is_empty());
    for cell in &demo_cells {
        assert_eq!(cell.created_by, system);
        assert_eq!(cell.modified_by, system);
    }
}

#[tokio::test]
async fn membership_changes_name_who_made_them() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let workspace = create_workspace(&app, "acme").await;
    let owner = join_workspace(&app, workspace, "owner", OWNER_ROLE).await;
    let invitee = create_user(&app, "invitee", PLATFORM_MEMBER).await;
    let invitee_session = sign_in(&app, &invitee).await;
    let membership = [
        ("tenant_id", workspace.into()),
        ("user_id", invitee.id.into()),
    ];

    let invited = owner
        .session
        .post(
            &format!("/tenant/{workspace}/invitation"),
            json!({ "email": invitee.email, "role_name": VIEWER_ROLE }),
        )
        .await;
    assert_eq!(invited.status, StatusCode::CREATED, "{}", invited.body);
    let accepted = invitee_session
        .post(
            &format!("/invitation/{}/accept", string_field(&invited.body, "id")),
            json!({}),
        )
        .await;
    assert_eq!(accepted.status, StatusCode::OK, "{}", accepted.body);

    let joined = attribution(&app, "tenant_membership", &membership).await;
    assert_eq!(joined.created_by, RecordedActor::person(invitee.id));
    assert_eq!(joined.modified_by, RecordedActor::person(invitee.id));

    let promoted = owner
        .session
        .put(
            &format!("/member/{}", invitee.id),
            json!({ "role_name": MEMBER_ROLE }),
        )
        .await;
    assert_eq!(promoted.status, StatusCode::OK, "{}", promoted.body);

    let promoted = attribution(&app, "tenant_membership", &membership).await;
    assert_eq!(promoted.created_by, RecordedActor::person(invitee.id));
    assert_eq!(promoted.modified_by, RecordedActor::person(owner.user.id));
}

#[tokio::test]
async fn a_new_workspace_names_its_creator_and_leaves_roles_to_the_system() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let administrator = admin_user_id(&app).await;

    let created = app.post("/tenant", json!({ "name": "Analytics" })).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    let workspace = uuid_field(&created.body, "id");

    assert_eq!(
        attribution_of(&app, "tenant", workspace).await.created_by,
        RecordedActor::person(administrator)
    );
    let roles = attributions(&app, "role", &[("tenant_id", workspace.into())]).await;
    assert!(!roles.is_empty());
    for role in &roles {
        assert_eq!(role.created_by, RecordedActor::system());
    }
    assert_eq!(
        attribution(
            &app,
            "tenant_membership",
            &[
                ("tenant_id", workspace.into()),
                ("user_id", administrator.into()),
            ],
        )
        .await
        .created_by,
        RecordedActor::person(administrator)
    );
}

#[tokio::test]
async fn content_edits_stamp_their_canvas() {
    let app = TestApp::spawn().await;
    let team = form_team(&app).await;
    team.editor
        .session
        .configure_live_ai_provider(LIVE_AI_PROVIDER)
        .await;
    let canvas = canvas_from_conversation(&team.alice.session, team.connection_id).await;
    let_bob_edit_the_conversation(&app, &team, &canvas.conversation_id).await;
    let alice = RecordedActor::person(team.alice.user.id);
    let alice_ai = RecordedActor::ai_for(team.alice.user.id);
    let bob_ai = RecordedActor::ai_for(team.bob.user.id);
    let editor = RecordedActor::person(team.editor.user.id);

    extend_canvas_through_the_agent(&team.bob.session, &canvas).await;
    assert_eq!(
        attribution_of(&app, "cell", canvas.sql_cell_id)
            .await
            .modified_by,
        bob_ai
    );
    assert_canvas_last_changed_by(&app, canvas.canvas_id, &bob_ai, "an AI edit").await;

    edit_cell_content(
        &team.alice.session,
        canvas.sql_cell_id,
        "SELECT order_status FROM orders",
    )
    .await;
    let edited_cell = attribution_of(&app, "cell", canvas.sql_cell_id).await;
    assert_eq!(
        edited_cell.modified_by, alice,
        "a person editing the cell after the AI takes it back"
    );
    assert_eq!(edited_cell.created_by, alice_ai);
    assert_canvas_last_changed_by(&app, canvas.canvas_id, &alice, "a human edit").await;

    reorder_cell(&team.editor.session, canvas.chart_cell_id, "UP").await;
    assert_canvas_last_changed_by(&app, canvas.canvas_id, &editor, "a reorder").await;

    update_dashboard(&team.alice.session, &canvas.dashboard).await;
    assert_canvas_last_changed_by(&app, canvas.canvas_id, &alice, "a dashboard edit").await;

    delete_cell(&team.editor.session, canvas.chart_cell_id).await;
    assert_canvas_last_changed_by(&app, canvas.canvas_id, &editor, "a delete").await;

    create_sql_cell(
        &team.alice.session,
        &canvas.notebook_id.to_string(),
        &team.connection_id.to_string(),
        "third",
        "SELECT 1",
    )
    .await;
    assert_canvas_last_changed_by(&app, canvas.canvas_id, &alice, "a create").await;

    assert_eq!(
        attribution_of(&app, "canvas", canvas.canvas_id)
            .await
            .created_by,
        alice_ai,
        "content edits never change who created the canvas"
    );
}

#[tokio::test]
async fn messages_stamp_their_conversation() {
    let app = TestApp::spawn().await;
    let team = form_team(&app).await;
    team.editor
        .session
        .configure_live_ai_provider(LIVE_AI_PROVIDER)
        .await;
    let session = &team.alice.session;
    let alice = RecordedActor::person(team.alice.user.id);
    let alice_ai = RecordedActor::ai_for(team.alice.user.id);

    let conversation_id = start_conversation(
        session,
        &team.connection_id.to_string(),
        ORDER_STATUS_QUESTION,
    )
    .await;
    let conversation = parse_uuid(&conversation_id);
    assert_conversation_last_changed_by(&app, conversation, &alice, "creation").await;

    let answer = run_completion(session, &conversation_id).await;
    assert_eq!(
        event_names(&answer).last().map(String::as_str),
        Some("completed")
    );
    wait_for_persisted_event(session, &conversation_id, "completed").await;
    assert_conversation_last_changed_by(&app, conversation, &alice_ai, "an answer").await;
    let answers = attributions(
        &app,
        "conversation_message",
        &[
            ("conversation_id", conversation.into()),
            ("sender", ASSISTANT_SENDER.into()),
        ],
    )
    .await;
    assert_eq!(answers.len(), 1);
    assert_eq!(answers[0].created_by, alice_ai);
    assert_eq!(answers[0].modified_by, alice_ai);

    change_visibility(
        session,
        CONVERSATION_RESOURCE,
        conversation,
        PRIVATE_VISIBILITY,
    )
    .await;
    assert_conversation_last_changed_by(&app, conversation, &alice, "a visibility change").await;

    run_canvas_command(session, &conversation_id).await;
    assert_conversation_last_changed_by(&app, conversation, &alice_ai, "a second answer").await;

    let follow_up = session
        .post(
            &format!("/conversational_analytics/{conversation_id}/messages"),
            json!({ "segments": [{ "type": "TEXT", "text": "And last month?" }] }),
        )
        .await;
    assert_eq!(follow_up.status, StatusCode::OK, "{}", follow_up.body);
    assert_conversation_last_changed_by(&app, conversation, &alice, "a follow-up question").await;

    assert_eq!(
        attribution_of(&app, "conversation", conversation)
            .await
            .created_by,
        alice
    );
}

#[tokio::test]
async fn responses_name_who_made_and_changed_the_row() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let admin = person_attribution(admin_user_id(&app).await, ADMIN_FULL_NAME);
    let system = system_attribution();

    let canvases = app.get("/canvas?page=0&page_size=50").await;
    assert_eq!(canvases.status, StatusCode::OK, "{}", canvases.body);
    let demo_canvas = named(&canvases.body["data"], DEMO_CANVAS_NAME);
    assert_eq!(demo_canvas["created_by"], system);
    assert_eq!(demo_canvas["modified_by"], system);
    let demo_canvas_id = string_field(demo_canvas, "id");

    let renamed = app
        .put(
            &format!("/canvas/{demo_canvas_id}"),
            json!({ "name": "Renamed", "description": null }),
        )
        .await;
    assert_eq!(renamed.status, StatusCode::OK, "{}", renamed.body);
    assert_eq!(renamed.body["created_by"], system);
    assert_eq!(renamed.body["modified_by"], admin);
    let fetched = app.get(&format!("/canvas/{demo_canvas_id}")).await;
    assert_eq!(fetched.body["modified_by"], admin);

    let created = app
        .post("/canvas", json!({ "name": "Mine", "description": null }))
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    assert_eq!(created.body["created_by"], admin);
    assert_eq!(created.body["modified_by"], admin);

    let notebooks = app.get(&format!("/notebook/canvas/{demo_canvas_id}")).await;
    assert_eq!(notebooks.status, StatusCode::OK, "{}", notebooks.body);
    let notebook = app
        .get(&format!(
            "/notebook/{}",
            string_field(&notebooks.body[0], "id")
        ))
        .await;
    assert_eq!(notebook.status, StatusCode::OK, "{}", notebook.body);
    let cells = notebook.body["cells"].as_array().expect("notebook cells");
    assert!(!cells.is_empty());
    assert!(cells.iter().all(|cell| cell["created_by"] == system));
    let mut edited_cell = cells[0].clone();
    edited_cell["name"] = json!("Edited");
    let edited_cell = app
        .put(
            &format!("/cell/{}", string_field(&cells[0], "id")),
            edited_cell,
        )
        .await;
    assert_eq!(edited_cell.status, StatusCode::OK, "{}", edited_cell.body);
    assert_eq!(edited_cell.body["created_by"], system);
    assert_eq!(edited_cell.body["modified_by"], admin);

    let connections = app.get("/connection").await;
    assert_eq!(connections.status, StatusCode::OK, "{}", connections.body);
    let demo_connection = named(&connections.body, DEMO_CONNECTION_NAME);
    assert_eq!(demo_connection["created_by"], system);
    let demo_connection_id = string_field(demo_connection, "id");
    let fetched_connection = app.get(&format!("/connection/{demo_connection_id}")).await;
    assert_eq!(fetched_connection.body["modified_by"], system);

    app.configure_live_ai_provider(LIVE_AI_PROVIDER).await;
    let conversation = app
        .post(
            "/conversational_analytics",
            json!({
                "connection_id": demo_connection_id,
                "segments": [{ "type": "TEXT", "text": "How many orders?" }],
            }),
        )
        .await;
    assert_eq!(
        conversation.status,
        StatusCode::CREATED,
        "{}",
        conversation.body
    );
    assert_eq!(conversation.body["created_by"], admin);
    let listed = app.get("/conversational_analytics").await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.body);
    assert_eq!(listed.body["items"][0]["modified_by"], admin);
}

#[tokio::test]
async fn created_by_me_includes_work_the_ai_did_for_me() {
    let app = TestApp::spawn().await;
    let team = form_team(&app).await;
    let alice_id = team.alice.user.id;
    let bob_id = team.bob.user.id;

    create_canvas(&team.alice.session, "alice by hand").await;
    for (name, actor) in [
        ("ai for alice", RecordedActor::ai_for(alice_id)),
        ("ai for bob in alice's", RecordedActor::ai_for(bob_id)),
    ] {
        let canvas = create_canvas(&team.alice.session, name).await;
        record_as_made_by(&app, "canvas", parse_uuid(&canvas.canvas_id), &actor).await;
    }
    create_canvas(&team.bob.session, "bob by hand").await;

    let all_canvases = team.alice.session.get("/canvas?page=0&page_size=50").await;
    assert_eq!(all_canvases.status, StatusCode::OK, "{}", all_canvases.body);
    assert_eq!(
        all_canvases.body["data"]
            .as_array()
            .expect("canvases")
            .len(),
        4
    );

    let mine = team
        .alice
        .session
        .get("/canvas?page=0&page_size=50&created_by=me")
        .await;
    assert_eq!(mine.status, StatusCode::OK, "{}", mine.body);
    assert_eq!(
        sorted_names(&mine.body["data"]),
        vec!["ai for alice", "alice by hand"]
    );
    assert_eq!(mine.body["total_items"], 2);

    for (name, actor) in [
        ("alice by hand", RecordedActor::person(alice_id)),
        ("ai for alice", RecordedActor::ai_for(alice_id)),
        ("ai for bob", RecordedActor::ai_for(bob_id)),
        ("startup", RecordedActor::system()),
    ] {
        let connection_id = uuid_field(
            &app.create_named_sqlite_demo_connection_as(&team.editor.session, name)
                .await,
            "id",
        );
        set_owner_and_visibility(
            &app,
            "connection",
            connection_id,
            alice_id,
            TENANT_VISIBILITY,
        )
        .await;
        record_as_made_by(&app, "connection", connection_id, &actor).await;
    }

    let my_connections = team.alice.session.get("/connection?created_by=me").await;
    assert_eq!(
        my_connections.status,
        StatusCode::OK,
        "{}",
        my_connections.body
    );
    assert_eq!(
        sorted_names(&my_connections.body),
        vec!["ai for alice", "alice by hand"]
    );
}

#[tokio::test]
async fn an_unknown_created_by_filter_is_rejected() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;

    assert_eq!(
        app.get("/canvas?created_by=me").await.status,
        StatusCode::OK
    );
    assert_eq!(app.get("/canvas?page=1").await.status, StatusCode::OK);
    for unknown_filter in ["/canvas?created_by=everyone", "/canvas?created_by="] {
        let refused = app.get(unknown_filter).await;
        assert_eq!(
            refused.status,
            StatusCode::BAD_REQUEST,
            "{unknown_filter}: {}",
            refused.body
        );
        assert_eq!(refused.body["error"], UNKNOWN_CREATED_BY_FILTER_MESSAGE);
    }
}

#[tokio::test]
async fn content_writes_take_the_canvas_first() {
    let app = TestApp::spawn().await;
    let team = form_team(&app).await;
    team.editor
        .session
        .configure_live_ai_provider(LIVE_AI_PROVIDER)
        .await;
    let canvas = canvas_from_conversation(&team.alice.session, team.connection_id).await;
    let_bob_edit_the_conversation(&app, &team, &canvas.conversation_id).await;
    record_write_order(&app).await;

    let third_cell_id = uuid_field(
        &create_sql_cell(
            &team.editor.session,
            &canvas.notebook_id.to_string(),
            &team.connection_id.to_string(),
            "third",
            "SELECT 1",
        )
        .await,
        "id",
    );
    assert_canvas_written_first(&app, "a cell create").await;

    edit_cell_content(
        &team.editor.session,
        canvas.sql_cell_id,
        "SELECT order_status FROM orders",
    )
    .await;
    assert_canvas_written_first(&app, "a cell edit").await;

    reorder_cell(&team.editor.session, third_cell_id, "TOP").await;
    assert_canvas_written_first(&app, "a reorder").await;

    update_dashboard(&team.editor.session, &canvas.dashboard).await;
    assert_canvas_written_first(&app, "a dashboard edit").await;

    extend_canvas_through_the_agent(&team.bob.session, &canvas).await;
    assert_canvas_written_first(&app, "an AI plan on an existing canvas").await;

    delete_cell(&team.alice.session, third_cell_id).await;
    assert_canvas_written_first(&app, "a cell delete").await;

    let deleted = team
        .alice
        .session
        .delete(&format!("/canvas/{}", canvas.canvas_id))
        .await;
    assert!(deleted.status.is_success(), "{}", deleted.body);
    assert_canvas_written_first(&app, "a canvas delete").await;
}

#[tokio::test]
async fn reordering_cells_keeps_who_last_edited_each_cell() {
    let app = TestApp::spawn().await;
    let team = form_team(&app).await;
    let canvas = create_canvas(&team.alice.session, "revenue").await;
    let canvas_id = parse_uuid(&canvas.canvas_id);
    let connection_id = team.connection_id.to_string();
    let mut cell_ids = Vec::new();
    for name in ["first", "second", "third"] {
        let cell = create_sql_cell(
            &team.alice.session,
            &canvas.notebook_id,
            &connection_id,
            name,
            "SELECT 1",
        )
        .await;
        cell_ids.push(uuid_field(&cell, "id"));
    }
    let [first_cell_id, second_cell_id, third_cell_id] = cell_ids[..] else {
        panic!("three cells were created");
    };
    record_as_last_changed_by(
        &app,
        "cell",
        first_cell_id,
        &RecordedActor::ai_for(team.bob.user.id),
    )
    .await;

    reorder_cell(&team.editor.session, third_cell_id, "TOP").await;

    assert_eq!(
        [
            display_order(&app, first_cell_id).await,
            display_order(&app, second_cell_id).await,
            display_order(&app, third_cell_id).await,
        ],
        [2, 3, 1]
    );
    assert_eq!(
        attribution_of(&app, "cell", first_cell_id)
            .await
            .modified_by,
        RecordedActor::ai_for(team.bob.user.id),
        "moving another cell keeps the AI's edit on this one"
    );
    assert_eq!(
        attribution_of(&app, "cell", second_cell_id)
            .await
            .modified_by,
        RecordedActor::person(team.alice.user.id)
    );
    assert_eq!(
        attribution_of(&app, "cell", third_cell_id)
            .await
            .modified_by,
        RecordedActor::person(team.alice.user.id),
        "moving a cell is not an edit of the cell"
    );
    assert_eq!(
        attribution_of(&app, "canvas", canvas_id).await.modified_by,
        RecordedActor::person(team.editor.user.id),
        "a reorder is a change to the canvas"
    );
}
