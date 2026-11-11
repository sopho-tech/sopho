use crate::authentication::oidc::service as oidc_service;
use crate::authentication::service as authentication_service;
use crate::authentication::service::{PasswordFlowError, UserStatus};
use crate::authorization::constants::{PlatformPermission, PlatformRole, SystemRole};
use crate::authorization::dto::{Actor, TenantId, UserId};
use crate::authorization::service as authorization_service;
use crate::common::AppState;
use crate::email_delivery::dto::EmailLogPageDto;
use crate::email_delivery::service as email_delivery_service;
use crate::email_template::service as email_template_service;
use crate::entity::user;
use crate::platform::constants::{FIRST_PAGE, PLATFORM_PAGE_SIZE};
use crate::platform::dto::{
    DeactivatedUserDto, EmailLogQuery, EmailTemplatePreviewDto, PlatformTenantDto,
    PlatformUserCapabilities, PlatformUserDto, PlatformUserPageDto, TenantSummaryDto,
    UserListQuery,
};
use crate::platform::error::PlatformError;
use crate::tenant::invitation::service as invitation_service;
use crate::tenant::membership::service as membership_service;
use crate::tenant::service as tenant_service;
use sea_orm::TransactionTrait;
use std::str::FromStr;
use uuid::Uuid;

pub async fn list_users(
    app_state: &AppState,
    actor: UserId,
    query: UserListQuery,
) -> Result<PlatformUserPageDto, PlatformError> {
    let actor_role =
        require_platform_permission(app_state, actor, PlatformPermission::UserManage).await?;
    let may_appoint_owners = authorization_service::has_platform_permission(
        actor_role,
        PlatformPermission::TenantAppointOwner,
    );
    let is_email_enabled = email_delivery_service::is_email_enabled(app_state);
    let active_administrator_count =
        authentication_service::count_active_platform_administrators(app_state).await?;
    let page = page_number(query.page)?;
    let user_page = authentication_service::list_users_page(
        app_state,
        search_text(query.search),
        page - 1,
        PLATFORM_PAGE_SIZE,
    )
    .await?;
    let user_ids: Vec<Uuid> = user_page.users.iter().map(|account| account.id).collect();
    let mut provider_names =
        oidc_service::provider_names_by_user(app_state, user_ids.clone()).await?;
    let mut memberships = tenant_service::memberships_by_user(app_state, user_ids)
        .await
        .map_err(PlatformError::Tenant)?;

    let users = user_page
        .users
        .into_iter()
        .map(|account| {
            let status = account_status(&account)?;
            let is_active = status == UserStatus::Active;
            let has_password = account.password_hash.is_some();
            let is_platform_administrator =
                account.platform_role == PlatformRole::Administrator.as_str();
            let is_current_user = account.id == actor.get_uuid();
            let is_last_active_administrator =
                is_active_platform_administrator(&account) && active_administrator_count <= 1;
            Ok(PlatformUserDto {
                capabilities: PlatformUserCapabilities {
                    can_deactivate: is_active
                        && account.id != actor.get_uuid()
                        && !is_last_active_administrator,
                    can_reactivate: !is_active,
                    can_send_password_reset: is_active && has_password && is_email_enabled,
                    can_appoint_owner: is_active && may_appoint_owners,
                    can_make_administrator: is_active && !is_platform_administrator,
                    can_remove_administrator: is_platform_administrator
                        && !is_current_user
                        && !is_last_active_administrator,
                },
                is_current_user,
                is_platform_administrator,
                status,
                has_password,
                provider_names: provider_names.remove(&account.id).unwrap_or_default(),
                tenants: memberships.remove(&account.id).unwrap_or_default(),
                id: account.id,
                full_name: account.full_name,
                email: account.email,
                created_at: account.created_at,
            })
        })
        .collect::<Result<Vec<_>, PlatformError>>()?;
    Ok(PlatformUserPageDto {
        users,
        page,
        page_size: PLATFORM_PAGE_SIZE,
        total: user_page.total,
        total_pages: user_page.total.div_ceil(PLATFORM_PAGE_SIZE),
    })
}

pub async fn deactivate_user(
    app_state: &AppState,
    actor: UserId,
    user_id: Uuid,
) -> Result<DeactivatedUserDto, PlatformError> {
    require_platform_permission(app_state, actor, PlatformPermission::UserManage).await?;
    let account = find_account(app_state, user_id).await?;
    if account.id == actor.get_uuid() {
        return Err(PlatformError::CannotDeactivateYourself);
    }
    if is_active_platform_administrator(&account)
        && authentication_service::count_active_platform_administrators(app_state).await? <= 1
    {
        return Err(PlatformError::LastPlatformAdministrator);
    }
    let owned_tenants: Vec<TenantSummaryDto> =
        tenant_service::memberships_by_user(app_state, vec![account.id])
            .await
            .map_err(PlatformError::Tenant)?
            .remove(&account.id)
            .unwrap_or_default()
            .into_iter()
            .filter(|membership| membership.role_name == SystemRole::Owner.as_str())
            .map(|membership| TenantSummaryDto {
                tenant_id: membership.tenant_id,
                tenant_name: membership.tenant_name,
            })
            .collect();

    let txn = app_state.database_connection.begin().await?;
    authentication_service::deactivate_user_transaction(&txn, account.id).await?;
    invitation_service::revoke_every_invitation_sent_by_transaction(&txn, account.id).await?;
    txn.commit().await?;

    let active_owner_counts = membership_service::active_owner_counts(app_state)
        .await
        .map_err(PlatformError::Membership)?;
    Ok(DeactivatedUserDto {
        tenants_without_active_owner: owned_tenants
            .into_iter()
            .filter(|owned_tenant| !active_owner_counts.contains_key(&owned_tenant.tenant_id))
            .collect(),
    })
}

pub async fn reactivate_user(
    app_state: &AppState,
    actor: UserId,
    user_id: Uuid,
) -> Result<(), PlatformError> {
    require_platform_permission(app_state, actor, PlatformPermission::UserManage).await?;
    let account = find_account(app_state, user_id).await?;
    authentication_service::reactivate_user(app_state, account.id).await?;
    Ok(())
}

pub async fn send_password_reset(
    app_state: &AppState,
    actor: UserId,
    user_id: Uuid,
) -> Result<(), PlatformError> {
    require_platform_permission(app_state, actor, PlatformPermission::UserManage).await?;
    let account = find_account(app_state, user_id).await?;
    if account.status != UserStatus::Active.as_str() {
        return Err(PlatformError::AccountDeactivated);
    }
    if account.password_hash.is_none() {
        return Err(PlatformError::PasswordReset(PasswordFlowError::NoPassword));
    }
    authentication_service::send_password_reset_email(app_state, &account, Some(actor.get_uuid()))
        .await
        .map_err(PlatformError::PasswordReset)
}

pub async fn make_administrator(
    app_state: &AppState,
    actor: UserId,
    user_id: Uuid,
) -> Result<(), PlatformError> {
    require_platform_permission(app_state, actor, PlatformPermission::UserManage).await?;
    let account = find_account(app_state, user_id).await?;
    if account.status != UserStatus::Active.as_str() {
        return Err(PlatformError::AccountDeactivated);
    }
    authentication_service::set_platform_role(app_state, account.id, PlatformRole::Administrator)
        .await?;
    Ok(())
}

pub async fn remove_administrator(
    app_state: &AppState,
    actor: UserId,
    user_id: Uuid,
) -> Result<(), PlatformError> {
    require_platform_permission(app_state, actor, PlatformPermission::UserManage).await?;
    let account = find_account(app_state, user_id).await?;
    if account.id == actor.get_uuid() {
        return Err(PlatformError::CannotRemoveYourOwnAdministratorRole);
    }
    if is_active_platform_administrator(&account)
        && authentication_service::count_active_platform_administrators(app_state).await? <= 1
    {
        return Err(PlatformError::LastPlatformAdministrator);
    }
    authentication_service::set_platform_role(app_state, account.id, PlatformRole::Member).await?;
    Ok(())
}

pub async fn list_email_template_previews(
    app_state: &AppState,
    actor: UserId,
) -> Result<Vec<EmailTemplatePreviewDto>, PlatformError> {
    require_platform_permission(app_state, actor, PlatformPermission::EmailManage).await?;
    Ok(email_template_service::render_sample_emails()?
        .into_iter()
        .map(|rendered_email| EmailTemplatePreviewDto {
            kind: rendered_email.kind,
            kind_display_name: rendered_email.kind.display_name().to_string(),
            sent_when: rendered_email.kind.sent_when().to_string(),
            subject: rendered_email.subject,
            html_body: rendered_email.html_body,
            text_body: rendered_email.text_body,
        })
        .collect())
}

pub async fn list_tenants(
    app_state: &AppState,
    actor: UserId,
) -> Result<Vec<PlatformTenantDto>, PlatformError> {
    require_platform_permission(app_state, actor, PlatformPermission::TenantAppointOwner).await?;
    let active_owner_counts = membership_service::active_owner_counts(app_state)
        .await
        .map_err(PlatformError::Membership)?;
    Ok(tenant_service::list_all_tenants(app_state)
        .await?
        .into_iter()
        .map(|found_tenant| PlatformTenantDto {
            active_owner_count: active_owner_counts
                .get(&found_tenant.id)
                .copied()
                .unwrap_or_default(),
            tenant_id: found_tenant.id,
            name: found_tenant.name,
            created_at: found_tenant.created_at,
        })
        .collect())
}

pub async fn appoint_owner(
    app_state: &AppState,
    actor: UserId,
    tenant_id: Uuid,
    user_id: Uuid,
) -> Result<(), PlatformError> {
    require_platform_permission(app_state, actor, PlatformPermission::TenantAppointOwner).await?;
    let appointed_tenant = tenant_service::find_tenant(app_state, tenant_id)
        .await?
        .ok_or(PlatformError::TenantNotFound)?;
    let appointee = find_account(app_state, user_id).await?;
    if appointee.status != UserStatus::Active.as_str() {
        return Err(PlatformError::AppointeeDeactivated);
    }
    membership_service::appoint_owner(
        app_state,
        &TenantId::from_system_unchecked(appointed_tenant.id),
        appointee.id,
        &Actor::human(actor),
    )
    .await
    .map_err(PlatformError::Membership)
}

pub async fn list_email_log(
    app_state: &AppState,
    actor: UserId,
    query: EmailLogQuery,
) -> Result<EmailLogPageDto, PlatformError> {
    require_platform_permission(app_state, actor, PlatformPermission::EmailManage).await?;
    email_delivery_service::list_email_log(
        app_state,
        query.status,
        search_text(query.search),
        page_number(query.page)?,
        PLATFORM_PAGE_SIZE,
    )
    .await
    .map_err(PlatformError::EmailDelivery)
}

pub async fn retry_email(
    app_state: &AppState,
    actor: UserId,
    email_id: Uuid,
) -> Result<(), PlatformError> {
    require_platform_permission(app_state, actor, PlatformPermission::EmailManage).await?;
    email_delivery_service::retry_email(app_state, email_id)
        .await
        .map_err(PlatformError::EmailDelivery)
}

async fn require_platform_permission(
    app_state: &AppState,
    actor: UserId,
    permission: PlatformPermission,
) -> Result<PlatformRole, PlatformError> {
    let platform_role = authentication_service::find_platform_role(app_state, actor)
        .await
        .map_err(|error| PlatformError::Internal(error.to_string()))?;
    authorization_service::require_platform_permission(platform_role, permission)?;
    Ok(platform_role)
}

fn is_active_platform_administrator(account: &user::Model) -> bool {
    account.platform_role == PlatformRole::Administrator.as_str()
        && account.status == UserStatus::Active.as_str()
}

async fn find_account(app_state: &AppState, user_id: Uuid) -> Result<user::Model, PlatformError> {
    authentication_service::find_optional_user(app_state, user_id)
        .await?
        .ok_or(PlatformError::UserNotFound)
}

fn account_status(account: &user::Model) -> Result<UserStatus, PlatformError> {
    UserStatus::from_str(&account.status).map_err(|_| {
        PlatformError::Internal(format!(
            "User {} has an unknown status {}",
            account.id, account.status
        ))
    })
}

fn page_number(requested_page: Option<u64>) -> Result<u64, PlatformError> {
    match requested_page {
        None => Ok(FIRST_PAGE),
        Some(page) if page >= FIRST_PAGE => Ok(page),
        Some(_) => Err(PlatformError::InvalidPage),
    }
}

fn search_text(search: Option<String>) -> Option<String> {
    search
        .map(|search| search.trim().to_string())
        .filter(|search| !search.is_empty())
}
