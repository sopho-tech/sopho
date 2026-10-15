use crate::support::canvas_builder::string_field;
use crate::support::constants::{
    ADMIN_ROLE, AUDITOR_ROLE, CANVAS_RESOURCE, CONNECTION_RESOURCE, CONVERSATION_RESOURCE,
    EDITOR_ACCESS, MEMBER_ROLE, OWNER_ROLE, PLATFORM_MEMBER, PRIVATE_VISIBILITY, TENANT_VISIBILITY,
    VIEWER_ACCESS, VIEWER_ROLE,
};
use crate::support::conversation::{
    answer_order_status_question, append_canvas_command, event_data, run_completion,
    wait_for_persisted_event,
};
use crate::support::records::{
    insert_canvas, insert_conversation, insert_grant, insert_notebook, insert_sql_cell,
    set_owner_and_visibility,
};
use crate::support::test_app::JsonResponse;
use crate::support::workspaces::{
    create_custom_role, create_user, create_workspace, join_workspace,
};
use crate::support::{TestApp, UserSession};
use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

fn canvas_path(canvas_id: Uuid) -> String {
    format!("/canvas/{canvas_id}")
}

fn sharing_path(resource_type: &str, resource_id: Uuid) -> String {
    format!("/grant/{resource_type}/{resource_id}")
}

fn uuid_field(value: &Value, field: &str) -> Uuid {
    string_field(value, field)
        .parse()
        .unwrap_or_else(|error| panic!("{field} is not a UUID: {error}"))
}

async fn rename_canvas(session: &UserSession, canvas_id: Uuid, name: &str) -> JsonResponse {
    session
        .put(
            &canvas_path(canvas_id),
            json!({ "name": name, "description": null }),
        )
        .await
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

async fn visible_canvas_names(session: &UserSession) -> Vec<String> {
    let page = session.get("/canvas?page=0&page_size=50").await;
    assert_eq!(page.status, StatusCode::OK, "{}", page.body);
    sorted_names(&page.body["data"])
}

async fn visible_notebook_names(session: &UserSession) -> Vec<String> {
    let page = session.get("/notebook?page=0&page_size=50").await;
    assert_eq!(page.status, StatusCode::OK, "{}", page.body);
    sorted_names(&page.body["data"])
}

async fn visible_connection_names(session: &UserSession) -> Vec<String> {
    let connections = session.get("/connection").await;
    assert_eq!(connections.status, StatusCode::OK, "{}", connections.body);
    sorted_names(&connections.body)
}

async fn search_result_names(session: &UserSession, query: &str, filter: &str) -> Vec<String> {
    let found = session
        .get(&format!("/search?query={query}&filters={filter}"))
        .await;
    assert_eq!(found.status, StatusCode::OK, "{}", found.body);
    sorted_names(&found.body["data"])
}

async fn named_connection(app: &TestApp, creator: &UserSession, name: &str) -> Uuid {
    let connection = app
        .create_named_sqlite_demo_connection_as(creator, name)
        .await;
    uuid_field(&connection, "id")
}

#[tokio::test]
async fn owner_can_read_and_write_own_private_canvas() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;
    let canvas_id = insert_canvas(&app, workspace, "own", member.user.id, PRIVATE_VISIBILITY).await;

    assert_eq!(
        member.session.get(&canvas_path(canvas_id)).await.status,
        StatusCode::OK
    );
    assert_eq!(
        rename_canvas(&member.session, canvas_id, "renamed")
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        member
            .session
            .get(&sharing_path(CANVAS_RESOURCE, canvas_id))
            .await
            .status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn member_cannot_reach_another_members_private_canvas() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;
    let canvas_id = insert_canvas(
        &app,
        workspace,
        "someone else's",
        Uuid::new_v4(),
        PRIVATE_VISIBILITY,
    )
    .await;

    assert_eq!(
        member.session.get(&canvas_path(canvas_id)).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        rename_canvas(&member.session, canvas_id, "renamed")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn admin_can_read_private_canvas_it_does_not_own() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let admin = join_workspace(&app, workspace, "admin", ADMIN_ROLE).await;
    let canvas_id = insert_canvas(
        &app,
        workspace,
        "someone else's",
        Uuid::new_v4(),
        PRIVATE_VISIBILITY,
    )
    .await;

    assert_eq!(
        admin.session.get(&canvas_path(canvas_id)).await.status,
        StatusCode::OK
    );
    assert_eq!(
        rename_canvas(&admin.session, canvas_id, "renamed")
            .await
            .status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn tenant_visibility_grants_read_but_not_write() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;
    let canvas_id =
        insert_canvas(&app, workspace, "shared", Uuid::new_v4(), TENANT_VISIBILITY).await;

    assert_eq!(
        member.session.get(&canvas_path(canvas_id)).await.status,
        StatusCode::OK
    );
    assert_eq!(
        rename_canvas(&member.session, canvas_id, "renamed")
            .await
            .status,
        StatusCode::NOT_FOUND,
        "tenant-visible must not mean tenant-editable"
    );
    assert_eq!(
        member.session.delete(&canvas_path(canvas_id)).await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn viewer_role_cannot_write_even_what_it_can_read() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let viewer = join_workspace(&app, workspace, "viewer", VIEWER_ROLE).await;
    let canvas_id =
        insert_canvas(&app, workspace, "shared", Uuid::new_v4(), TENANT_VISIBILITY).await;

    assert_eq!(
        viewer.session.get(&canvas_path(canvas_id)).await.status,
        StatusCode::OK
    );
    let write = rename_canvas(&viewer.session, canvas_id, "renamed").await;
    assert_eq!(
        write.status,
        StatusCode::FORBIDDEN,
        "a viewer must fail the capability gate, not the reachability gate: {}",
        write.body
    );
    assert_eq!(write.body["error"], "Permission denied: canvas:write");
}

#[tokio::test]
async fn viewer_grant_allows_read_only_and_editor_grant_allows_write() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let reader = join_workspace(&app, workspace, "reader", MEMBER_ROLE).await;
    let editor = join_workspace(&app, workspace, "editor", MEMBER_ROLE).await;
    let read_canvas_id =
        insert_canvas(&app, workspace, "read", Uuid::new_v4(), PRIVATE_VISIBILITY).await;
    insert_grant(
        &app,
        workspace,
        CANVAS_RESOURCE,
        read_canvas_id,
        reader.user.id,
        VIEWER_ACCESS,
    )
    .await;
    let write_canvas_id =
        insert_canvas(&app, workspace, "write", Uuid::new_v4(), PRIVATE_VISIBILITY).await;
    insert_grant(
        &app,
        workspace,
        CANVAS_RESOURCE,
        write_canvas_id,
        editor.user.id,
        EDITOR_ACCESS,
    )
    .await;

    assert_eq!(
        reader
            .session
            .get(&canvas_path(read_canvas_id))
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        rename_canvas(&reader.session, read_canvas_id, "renamed")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        editor
            .session
            .get(&canvas_path(write_canvas_id))
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        rename_canvas(&editor.session, write_canvas_id, "renamed")
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        editor
            .session
            .get(&sharing_path(CANVAS_RESOURCE, write_canvas_id))
            .await
            .status,
        StatusCode::NOT_FOUND,
        "an editor grantee must not be able to re-share"
    );
}

#[tokio::test]
async fn subject_from_another_tenant_is_rejected_before_any_gate() {
    let app = TestApp::spawn().await;
    let first = create_workspace(&app, "first").await;
    let second = create_workspace(&app, "second").await;
    let owner = join_workspace(&app, first, "owner", OWNER_ROLE).await;
    let canvas_id = insert_canvas(&app, second, "theirs", owner.user.id, TENANT_VISIBILITY).await;

    assert_eq!(
        owner.session.get(&canvas_path(canvas_id)).await.status,
        StatusCode::NOT_FOUND,
        "owning the resource must not override workspace isolation"
    );
}

#[tokio::test]
async fn list_returns_exactly_what_the_gate_would_allow() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;
    insert_canvas(
        &app,
        workspace,
        "own private",
        member.user.id,
        PRIVATE_VISIBILITY,
    )
    .await;
    insert_canvas(
        &app,
        workspace,
        "tenant visible",
        Uuid::new_v4(),
        TENANT_VISIBILITY,
    )
    .await;
    insert_canvas(
        &app,
        workspace,
        "someone elses private",
        Uuid::new_v4(),
        PRIVATE_VISIBILITY,
    )
    .await;
    let granted_canvas_id = insert_canvas(
        &app,
        workspace,
        "granted private",
        Uuid::new_v4(),
        PRIVATE_VISIBILITY,
    )
    .await;
    insert_grant(
        &app,
        workspace,
        CANVAS_RESOURCE,
        granted_canvas_id,
        member.user.id,
        VIEWER_ACCESS,
    )
    .await;

    assert_eq!(
        visible_canvas_names(&member.session).await,
        vec!["granted private", "own private", "tenant visible"],
        "the list must show owned, tenant-visible and granted canvases only"
    );
}

#[tokio::test]
async fn list_shows_everything_to_a_role_that_can_read_private() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let admin = join_workspace(&app, workspace, "admin", ADMIN_ROLE).await;
    insert_canvas(
        &app,
        workspace,
        "a private",
        Uuid::new_v4(),
        PRIVATE_VISIBILITY,
    )
    .await;
    insert_canvas(
        &app,
        workspace,
        "b tenant",
        Uuid::new_v4(),
        TENANT_VISIBILITY,
    )
    .await;

    assert_eq!(
        visible_canvas_names(&admin.session).await,
        vec!["a private", "b tenant"]
    );
}

#[tokio::test]
async fn list_never_crosses_tenants() {
    let app = TestApp::spawn().await;
    let first = create_workspace(&app, "first").await;
    let second = create_workspace(&app, "second").await;
    let member = join_workspace(&app, first, "member", MEMBER_ROLE).await;
    let admin = join_workspace(&app, first, "admin", ADMIN_ROLE).await;
    insert_canvas(&app, first, "mine", member.user.id, PRIVATE_VISIBILITY).await;
    insert_canvas(&app, second, "theirs", Uuid::new_v4(), TENANT_VISIBILITY).await;

    assert_eq!(visible_canvas_names(&member.session).await, vec!["mine"]);
    assert_eq!(
        visible_canvas_names(&admin.session).await,
        vec!["mine"],
        "read_private must not lift the workspace boundary"
    );
}

#[tokio::test]
async fn capabilities_reflect_what_the_gates_allow() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;
    let viewer = join_workspace(&app, workspace, "viewer", VIEWER_ROLE).await;
    let owned_canvas_id =
        insert_canvas(&app, workspace, "owned", member.user.id, PRIVATE_VISIBILITY).await;
    let shared_canvas_id =
        insert_canvas(&app, workspace, "shared", Uuid::new_v4(), TENANT_VISIBILITY).await;

    let owned = member.session.get(&canvas_path(owned_canvas_id)).await;
    assert_eq!(
        owned.body["capabilities"],
        json!({ "can_edit": true, "can_delete": true, "can_share": true })
    );
    let readable = member.session.get(&canvas_path(shared_canvas_id)).await;
    assert_eq!(
        readable.body["capabilities"],
        json!({ "can_edit": false, "can_delete": false, "can_share": false }),
        "tenant-visible is readable but not editable"
    );
    let viewed = viewer.session.get(&canvas_path(shared_canvas_id)).await;
    assert_eq!(
        viewed.body["capabilities"],
        json!({ "can_edit": false, "can_delete": false, "can_share": false })
    );
}

#[tokio::test]
async fn conversation_and_connection_dtos_carry_capabilities() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let administrator = join_workspace(&app, workspace, "administrator", ADMIN_ROLE).await;
    let owner = join_workspace(&app, workspace, "owner", MEMBER_ROLE).await;
    let reader = join_workspace(&app, workspace, "reader", MEMBER_ROLE).await;
    let connection_id = named_connection(&app, &administrator.session, "shared connection").await;
    set_owner_and_visibility(
        &app,
        "connection",
        connection_id,
        owner.user.id,
        TENANT_VISIBILITY,
    )
    .await;
    let conversation_id = insert_conversation(
        &app,
        workspace,
        "shared conversation",
        owner.user.id,
        TENANT_VISIBILITY,
        connection_id,
    )
    .await;
    let conversation_path = format!("/conversational_analytics/{conversation_id}");
    let connection_path = format!("/connection/{connection_id}");

    let owner_conversation = owner.session.get(&conversation_path).await;
    let owner_capabilities = &owner_conversation.body["conversation"]["capabilities"];
    assert_eq!(owner_capabilities["can_edit"], true);
    assert_eq!(owner_capabilities["can_share"], true);

    let reader_conversation = reader.session.get(&conversation_path).await;
    let reader_capabilities = &reader_conversation.body["conversation"]["capabilities"];
    assert_eq!(
        reader_capabilities["can_edit"], false,
        "tenant-visible is readable but not editable"
    );
    assert_eq!(reader_capabilities["can_share"], false);

    let owner_connection = owner.session.get(&connection_path).await;
    assert_eq!(
        owner_connection.body["capabilities"]["can_edit"], false,
        "a member holds connection:read but not connection:write, so owning one does not grant edit"
    );
    let administrator_connection = administrator.session.get(&connection_path).await;
    assert_eq!(
        administrator_connection.body["capabilities"]["can_edit"],
        true
    );
}

#[tokio::test]
async fn created_and_updated_resources_carry_the_callers_capabilities() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;
    let administrator = join_workspace(&app, workspace, "administrator", ADMIN_ROLE).await;

    let created_canvas = member
        .session
        .post("/canvas", json!({ "name": "draft", "description": null }))
        .await;
    assert_eq!(
        created_canvas.status,
        StatusCode::CREATED,
        "{}",
        created_canvas.body
    );
    assert_eq!(created_canvas.body["capabilities"]["can_share"], true);

    let renamed_canvas = rename_canvas(
        &member.session,
        uuid_field(&created_canvas.body, "id"),
        "final",
    )
    .await;
    assert_eq!(
        renamed_canvas.status,
        StatusCode::OK,
        "{}",
        renamed_canvas.body
    );
    assert_eq!(renamed_canvas.body["capabilities"]["can_edit"], true);
    assert_eq!(renamed_canvas.body["capabilities"]["can_share"], true);

    let created_connection = app
        .create_named_sqlite_demo_connection_as(&administrator.session, "warehouse")
        .await;
    assert_eq!(created_connection["capabilities"]["can_edit"], true);

    let conversation_id = insert_conversation(
        &app,
        workspace,
        "question",
        member.user.id,
        PRIVATE_VISIBILITY,
        uuid_field(&created_connection, "id"),
    )
    .await;
    let conversation_path = format!("/conversational_analytics/{conversation_id}");
    let mut conversation =
        member.session.get(&conversation_path).await.body["conversation"].clone();
    conversation["name"] = json!("renamed question");
    let renamed_conversation = member.session.put(&conversation_path, conversation).await;
    assert_eq!(
        renamed_conversation.status,
        StatusCode::OK,
        "{}",
        renamed_conversation.body
    );
    assert_eq!(renamed_conversation.body["capabilities"]["can_share"], true);
}

#[tokio::test]
async fn a_child_resource_inherits_its_roots_decision() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let owner = join_workspace(&app, workspace, "owner", MEMBER_ROLE).await;
    let other = join_workspace(&app, workspace, "other", MEMBER_ROLE).await;
    let canvas_id = insert_canvas(
        &app,
        workspace,
        "private canvas",
        owner.user.id,
        PRIVATE_VISIBILITY,
    )
    .await;
    let notebook_id = insert_notebook(&app, workspace, canvas_id, "Notebook").await;
    let notebook_path = format!("/notebook/{notebook_id}");

    assert_eq!(
        owner.session.get(&notebook_path).await.status,
        StatusCode::OK,
        "the canvas owner must reach a notebook inside it"
    );
    assert_eq!(
        other.session.get(&notebook_path).await.status,
        StatusCode::NOT_FOUND,
        "a notebook must not be reachable when its canvas is not"
    );
}

#[tokio::test]
async fn notebook_list_inherits_the_reachability_of_its_canvas() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;
    let own_canvas_id = insert_canvas(
        &app,
        workspace,
        "own canvas",
        member.user.id,
        PRIVATE_VISIBILITY,
    )
    .await;
    let tenant_canvas_id = insert_canvas(
        &app,
        workspace,
        "tenant canvas",
        Uuid::new_v4(),
        TENANT_VISIBILITY,
    )
    .await;
    let foreign_canvas_id = insert_canvas(
        &app,
        workspace,
        "someone else's canvas",
        Uuid::new_v4(),
        PRIVATE_VISIBILITY,
    )
    .await;
    let granted_canvas_id = insert_canvas(
        &app,
        workspace,
        "granted canvas",
        Uuid::new_v4(),
        PRIVATE_VISIBILITY,
    )
    .await;
    insert_grant(
        &app,
        workspace,
        CANVAS_RESOURCE,
        granted_canvas_id,
        member.user.id,
        VIEWER_ACCESS,
    )
    .await;
    insert_notebook(&app, workspace, own_canvas_id, "a own").await;
    insert_notebook(&app, workspace, tenant_canvas_id, "b tenant").await;
    insert_notebook(&app, workspace, foreign_canvas_id, "c foreign").await;
    insert_notebook(&app, workspace, granted_canvas_id, "d granted").await;

    assert_eq!(
        visible_notebook_names(&member.session).await,
        vec!["a own", "b tenant", "d granted"],
        "a notebook must be listed only when its canvas is reachable"
    );
    let admin = join_workspace(&app, workspace, "admin", ADMIN_ROLE).await;
    assert_eq!(
        visible_notebook_names(&admin.session).await,
        vec!["a own", "b tenant", "c foreign", "d granted"],
        "read_private on the parent canvas must reach every notebook"
    );
}

#[tokio::test]
async fn notebook_list_never_crosses_tenants() {
    let app = TestApp::spawn().await;
    let first = create_workspace(&app, "first").await;
    let second = create_workspace(&app, "second").await;
    let admin = join_workspace(&app, first, "admin", ADMIN_ROLE).await;
    let mine = insert_canvas(&app, first, "mine", Uuid::new_v4(), TENANT_VISIBILITY).await;
    let theirs = insert_canvas(&app, second, "theirs", Uuid::new_v4(), TENANT_VISIBILITY).await;
    insert_notebook(&app, first, mine, "mine").await;
    insert_notebook(&app, second, theirs, "theirs").await;

    assert_eq!(
        visible_notebook_names(&admin.session).await,
        vec!["mine"],
        "read_private must not lift the workspace boundary"
    );
}

#[tokio::test]
async fn connection_list_returns_exactly_what_the_gate_would_allow() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let administrator = join_workspace(&app, workspace, "administrator", ADMIN_ROLE).await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;
    for (name, owner_user_id, visibility) in [
        ("a own private", member.user.id, PRIVATE_VISIBILITY),
        ("b tenant", Uuid::new_v4(), TENANT_VISIBILITY),
        ("c foreign private", Uuid::new_v4(), PRIVATE_VISIBILITY),
        ("d granted private", Uuid::new_v4(), PRIVATE_VISIBILITY),
    ] {
        let connection_id = named_connection(&app, &administrator.session, name).await;
        set_owner_and_visibility(&app, "connection", connection_id, owner_user_id, visibility)
            .await;
        if name == "d granted private" {
            insert_grant(
                &app,
                workspace,
                CONNECTION_RESOURCE,
                connection_id,
                member.user.id,
                VIEWER_ACCESS,
            )
            .await;
        }
    }

    assert_eq!(
        visible_connection_names(&member.session).await,
        vec!["a own private", "b tenant", "d granted private"]
    );
    assert_eq!(
        visible_connection_names(&administrator.session).await,
        vec![
            "a own private",
            "b tenant",
            "c foreign private",
            "d granted private"
        ]
    );
}

#[tokio::test]
async fn connection_list_never_crosses_tenants() {
    let app = TestApp::spawn().await;
    let first = create_workspace(&app, "first").await;
    let second = create_workspace(&app, "second").await;
    let first_administrator = join_workspace(&app, first, "first-admin", ADMIN_ROLE).await;
    let second_administrator = join_workspace(&app, second, "second-admin", ADMIN_ROLE).await;
    let mine = named_connection(&app, &first_administrator.session, "mine").await;
    set_owner_and_visibility(&app, "connection", mine, Uuid::new_v4(), PRIVATE_VISIBILITY).await;
    named_connection(&app, &second_administrator.session, "theirs").await;

    assert_eq!(
        visible_connection_names(&first_administrator.session).await,
        vec!["mine"],
        "read_private must not lift the workspace boundary"
    );
}

#[tokio::test]
async fn search_does_not_surface_cells_of_unreachable_canvases() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;
    let own_canvas_id = insert_canvas(
        &app,
        workspace,
        "own canvas",
        member.user.id,
        PRIVATE_VISIBILITY,
    )
    .await;
    let foreign_canvas_id = insert_canvas(
        &app,
        workspace,
        "foreign canvas",
        Uuid::new_v4(),
        PRIVATE_VISIBILITY,
    )
    .await;
    let own_notebook_id = insert_notebook(&app, workspace, own_canvas_id, "notebook").await;
    let foreign_notebook_id = insert_notebook(&app, workspace, foreign_canvas_id, "notebook").await;
    insert_sql_cell(
        &app,
        workspace,
        own_notebook_id,
        "reachable query",
        "SELECT 1",
    )
    .await;
    insert_sql_cell(
        &app,
        workspace,
        foreign_notebook_id,
        "private query",
        "SELECT 1",
    )
    .await;

    assert_eq!(
        search_result_names(&member.session, "query", "sql_cells").await,
        vec!["reachable query"],
        "search must not reveal cells living in a canvas the searcher cannot reach"
    );
    let admin = join_workspace(&app, workspace, "admin", ADMIN_ROLE).await;
    assert_eq!(
        search_result_names(&admin.session, "query", "sql_cells").await,
        vec!["private query", "reachable query"],
        "read_private must reach cells in every canvas of the workspace"
    );
}

#[tokio::test]
async fn search_does_not_surface_unreachable_conversations() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;
    for (name, owner_user_id, visibility) in [
        ("mine", member.user.id, PRIVATE_VISIBILITY),
        ("theirs", Uuid::new_v4(), PRIVATE_VISIBILITY),
        ("shared", Uuid::new_v4(), TENANT_VISIBILITY),
    ] {
        insert_conversation(
            &app,
            workspace,
            name,
            owner_user_id,
            visibility,
            Uuid::new_v4(),
        )
        .await;
    }

    assert_eq!(
        search_result_names(&member.session, "", "conversations").await,
        vec!["mine", "shared"]
    );
}

#[tokio::test]
async fn agent_created_canvas_inherits_conversation_owner_and_visibility() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let administrator = join_workspace(&app, workspace, "administrator", ADMIN_ROLE).await;
    let owner = join_workspace(&app, workspace, "owner", MEMBER_ROLE).await;
    let other = join_workspace(&app, workspace, "other", MEMBER_ROLE).await;
    administrator
        .session
        .configure_live_ai_provider("anthropic")
        .await;
    let connection_id = named_connection(&app, &administrator.session, "connection").await;
    set_owner_and_visibility(
        &app,
        "connection",
        connection_id,
        owner.user.id,
        TENANT_VISIBILITY,
    )
    .await;
    let conversation_id =
        answer_order_status_question(&owner.session, &connection_id.to_string()).await;
    wait_for_persisted_event(&owner.session, &conversation_id, "completed").await;
    let conversation_uuid: Uuid = conversation_id.parse().expect("conversation id");
    set_owner_and_visibility(
        &app,
        "conversation",
        conversation_uuid,
        owner.user.id,
        PRIVATE_VISIBILITY,
    )
    .await;
    insert_grant(
        &app,
        workspace,
        CONVERSATION_RESOURCE,
        conversation_uuid,
        other.user.id,
        EDITOR_ACCESS,
    )
    .await;

    append_canvas_command(&other.session, &conversation_id).await;
    let events = run_completion(&other.session, &conversation_id).await;
    let canvas_id = uuid_field(&event_data(&events, "canvas_generated"), "canvas_id");

    let sharing = owner
        .session
        .get(&sharing_path(CANVAS_RESOURCE, canvas_id))
        .await;
    assert_eq!(sharing.status, StatusCode::OK, "{}", sharing.body);
    assert_eq!(
        sharing.body["owner"]["user_id"],
        owner.user.id.to_string(),
        "the canvas must belong to the conversation owner, not whoever asked the question"
    );
    assert_eq!(
        sharing.body["visibility"], PRIVATE_VISIBILITY,
        "a canvas generated from a private conversation must not become workspace visible"
    );
    assert_eq!(
        other.session.get(&canvas_path(canvas_id)).await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn granted_editor_cannot_reshare_canvas() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let owner = join_workspace(&app, workspace, "owner", MEMBER_ROLE).await;
    let editor = join_workspace(&app, workspace, "editor", MEMBER_ROLE).await;
    let canvas_id = insert_canvas(
        &app,
        workspace,
        "private",
        owner.user.id,
        PRIVATE_VISIBILITY,
    )
    .await;
    insert_grant(
        &app,
        workspace,
        CANVAS_RESOURCE,
        canvas_id,
        editor.user.id,
        EDITOR_ACCESS,
    )
    .await;

    assert_eq!(
        editor.session.get(&canvas_path(canvas_id)).await.status,
        StatusCode::OK
    );
    assert_eq!(
        rename_canvas(&editor.session, canvas_id, "renamed")
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        editor
            .session
            .get(&sharing_path(CANVAS_RESOURCE, canvas_id))
            .await
            .status,
        StatusCode::NOT_FOUND,
        "re-sharing stays with the owner or an administrator"
    );
}

#[tokio::test]
async fn custom_role_with_read_private_can_see_private_canvas() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    create_custom_role(
        &app,
        workspace,
        AUDITOR_ROLE,
        &["canvas:read", "canvas:read_private"],
    )
    .await;
    let auditor = join_workspace(&app, workspace, "auditor", AUDITOR_ROLE).await;
    let canvas_id = insert_canvas(
        &app,
        workspace,
        "someone else's",
        Uuid::new_v4(),
        PRIVATE_VISIBILITY,
    )
    .await;

    assert_eq!(
        auditor.session.get(&canvas_path(canvas_id)).await.status,
        StatusCode::OK,
        "a custom role holding canvas:read_private reaches another member's private canvas"
    );
    assert_eq!(
        rename_canvas(&auditor.session, canvas_id, "renamed")
            .await
            .status,
        StatusCode::FORBIDDEN,
        "reading private does not imply writing"
    );
}

#[tokio::test]
async fn only_someone_who_can_share_may_change_visibility() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let owner = join_workspace(&app, workspace, "owner", MEMBER_ROLE).await;
    let other = join_workspace(&app, workspace, "other", MEMBER_ROLE).await;
    let canvas_id = insert_canvas(
        &app,
        workspace,
        "private",
        owner.user.id,
        PRIVATE_VISIBILITY,
    )
    .await;
    let visibility_path = format!("{}/visibility", sharing_path(CANVAS_RESOURCE, canvas_id));
    let widen = json!({ "visibility": TENANT_VISIBILITY });

    assert_eq!(
        other
            .session
            .put(&visibility_path, widen.clone())
            .await
            .status,
        StatusCode::NOT_FOUND,
        "a member who cannot reach the canvas must not be able to widen it"
    );
    let shared = owner.session.put(&visibility_path, widen).await;
    assert_eq!(shared.status, StatusCode::OK, "{}", shared.body);
    assert_eq!(shared.body["visibility"], TENANT_VISIBILITY);
    assert_eq!(
        other.session.get(&canvas_path(canvas_id)).await.status,
        StatusCode::OK,
        "once workspace-visible, every member can read it"
    );
}

#[tokio::test]
async fn sharing_with_someone_outside_the_tenant_is_rejected() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let owner = join_workspace(&app, workspace, "owner", MEMBER_ROLE).await;
    let canvas_id = insert_canvas(
        &app,
        workspace,
        "private",
        owner.user.id,
        PRIVATE_VISIBILITY,
    )
    .await;
    let outsider = create_user(&app, "outsider", PLATFORM_MEMBER).await;

    let shared = owner
        .session
        .post(
            "/grant",
            json!({
                "resource_type": CANVAS_RESOURCE,
                "resource_id": canvas_id,
                "principal_user_id": outsider.id,
                "access_level": VIEWER_ACCESS,
            }),
        )
        .await;

    assert_eq!(shared.status, StatusCode::BAD_REQUEST, "{}", shared.body);
}

#[tokio::test]
async fn sharing_state_names_the_owner_and_each_grantee() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "t").await;
    let owner = join_workspace(&app, workspace, "owner", MEMBER_ROLE).await;
    let grantee = join_workspace(&app, workspace, "grantee", MEMBER_ROLE).await;
    let canvas_id = insert_canvas(
        &app,
        workspace,
        "private",
        owner.user.id,
        PRIVATE_VISIBILITY,
    )
    .await;
    let granted = owner
        .session
        .post(
            "/grant",
            json!({
                "resource_type": CANVAS_RESOURCE,
                "resource_id": canvas_id,
                "principal_user_id": grantee.user.id,
                "access_level": EDITOR_ACCESS,
            }),
        )
        .await;
    assert_eq!(granted.status, StatusCode::CREATED, "{}", granted.body);

    let sharing = owner
        .session
        .get(&sharing_path(CANVAS_RESOURCE, canvas_id))
        .await;

    assert_eq!(sharing.status, StatusCode::OK, "{}", sharing.body);
    assert_eq!(sharing.body["visibility"], PRIVATE_VISIBILITY);
    assert_eq!(sharing.body["owner"]["full_name"], owner.user.full_name);
    let grants = sharing.body["grants"].as_array().expect("grants");
    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0]["access_level"], EDITOR_ACCESS);
    assert_eq!(grants[0]["principal"]["full_name"], grantee.user.full_name);
}
