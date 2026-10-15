use crate::authorization::constants::{PlatformRole, SystemRole};
use crate::authorization::dto::{Actor, TenantContext, TenantId, UserId};
use crate::common::AppState;
use crate::common::time_utils;
use crate::entity;
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

pub struct TenantFixture {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
}

impl TenantFixture {
    pub fn scope(&self) -> TenantId {
        TenantId::from_system_unchecked(self.tenant_id)
    }

    pub fn user(&self) -> UserId {
        UserId::from_system_unchecked(self.user_id)
    }
}

pub async fn insert_user(app_state: &AppState, label: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    entity::user::ActiveModel {
        id: Set(user_id),
        username: Set(format!("{}_{}", label, user_id)),
        password_hash: Set(None),
        email: Set(format!("{}_{}@example.test", label, user_id)),
        full_name: Set(label.to_string()),
        platform_role: Set(PlatformRole::Member.as_str().to_string()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(&app_state.database_connection)
    .await
    .unwrap();
    user_id
}

pub async fn create_tenant(app_state: &AppState, name: &str) -> Uuid {
    crate::tenant::service::create_tenant(app_state, name.to_string(), &Actor::system())
        .await
        .unwrap()
        .id
}

pub async fn add_member(app_state: &AppState, tenant_id: Uuid, user_id: Uuid, role: SystemRole) {
    crate::tenant::membership::service::add_member_with_role_name(
        app_state,
        &TenantId::from_system_unchecked(tenant_id),
        UserId::from_system_unchecked(user_id),
        role.as_str(),
        &Actor::system(),
    )
    .await
    .unwrap();
}

pub fn human_actor(user_id: Uuid) -> Actor {
    Actor::human(UserId::from_system_unchecked(user_id))
}

pub async fn resolve_context(
    app_state: &AppState,
    tenant_id: Uuid,
    user_id: Uuid,
) -> TenantContext {
    crate::authorization::service::resolve_tenant_context(
        app_state,
        UserId::from_system_unchecked(user_id),
        tenant_id,
    )
    .await
    .unwrap()
}

pub async fn context_with_role(
    app_state: &AppState,
    tenant_id: Uuid,
    role: SystemRole,
    label: &str,
) -> TenantContext {
    let user_id = insert_user(app_state, label).await;
    add_member(app_state, tenant_id, user_id, role).await;
    resolve_context(app_state, tenant_id, user_id).await
}

pub async fn create_tenant_with_owner(app_state: &AppState, tenant_name: &str) -> TenantFixture {
    let tenant_id = create_tenant(app_state, tenant_name).await;
    let user_id = insert_user(app_state, tenant_name).await;
    add_member(app_state, tenant_id, user_id, SystemRole::Owner).await;
    TenantFixture { tenant_id, user_id }
}
