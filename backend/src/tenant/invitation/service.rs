use crate::authentication::service as authentication_service;
use crate::authorization::constants::Permission;
use crate::authorization::dto::{Actor, TenantContext, TenantId, UserId};
use crate::authorization::service as authorization_service;
use crate::common::time_utils;
use crate::common::AppState;
use crate::entity::tenant_invitation;
use crate::tenant::invitation::constants::{InvitationStatus, INVITATION_EXPIRY_DAYS};
use crate::tenant::invitation::dto::{
    AcceptedInvitationDto, CreateInvitationDto, InvitationDto, ReceivedInvitationDto,
};
use crate::tenant::invitation::error::InvitationError;
use crate::tenant::invitation::repository;
use crate::tenant::invitation::repository::NewInvitation;
use crate::tenant::membership::service as membership_service;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::TransactionTrait;
use std::collections::HashMap;
use uuid::Uuid;

pub async fn create_invitation(
    app_state: &AppState,
    tenant_context: &TenantContext,
    payload: CreateInvitationDto,
) -> Result<InvitationDto, InvitationError> {
    authorization_service::require_capability(tenant_context, Permission::MemberInvite)?;
    if app_state.config.oidc_providers.is_empty() {
        return Err(InvitationError::SingleSignOnRequired);
    }
    let email = authentication_service::normalize_email(&payload.email);
    if !is_valid_email(&email) {
        return Err(InvitationError::InvalidEmail);
    }
    let role =
        membership_service::require_grantable_role(app_state, tenant_context, &payload.role_name)
            .await?;

    let tenant_id = tenant_context.tenant_id();
    if let Some(existing_user) =
        authentication_service::find_optional_user_by_email(app_state, &email).await?
    {
        if membership_service::is_active_member(app_state, tenant_id, existing_user.id).await? {
            return Err(InvitationError::AlreadyMember);
        }
    }
    if repository::find_unexpired_pending_invitation(
        &app_state.database_connection,
        tenant_id,
        &email,
        time_utils::now_utc_into(),
    )
    .await?
    .is_some()
    {
        return Err(InvitationError::AlreadyInvited);
    }

    let inviter =
        authentication_service::find_user_by_id(app_state, tenant_context.user_id()).await?;
    let invitation = repository::insert_invitation(
        &app_state.database_connection,
        tenant_id,
        NewInvitation {
            email: &email,
            role_id: role.id,
            expires_at: time_utils::now_plus_days_into(INVITATION_EXPIRY_DAYS),
            invited_by_user_id: inviter.id,
        },
    )
    .await?;

    Ok(InvitationDto {
        id: invitation.id,
        email: invitation.email,
        role_name: role.name,
        status: InvitationStatus::Pending,
        invited_by_name: inviter.full_name,
        expires_at: invitation.expires_at,
    })
}

pub async fn list_tenant_invitations(
    app_state: &AppState,
    tenant_context: &TenantContext,
) -> Result<Vec<InvitationDto>, InvitationError> {
    authorization_service::require_capability(tenant_context, Permission::MemberInvite)?;
    let invitations = repository::list_pending_invitations(
        &app_state.database_connection,
        tenant_context.tenant_id(),
    )
    .await?;
    let role_names = role_names_by_id(app_state, &invitations).await?;
    let inviter_names = inviter_names_by_id(app_state, &invitations).await?;

    let mut invitation_dtos = invitations
        .into_iter()
        .map(|invitation| {
            Ok(InvitationDto {
                id: invitation.id,
                status: effective_status(&invitation),
                role_name: lookup(&role_names, invitation.role_id, "role")?,
                invited_by_name: lookup(&inviter_names, invitation.invited_by_user_id, "inviter")?,
                email: invitation.email,
                expires_at: invitation.expires_at,
            })
        })
        .collect::<Result<Vec<_>, InvitationError>>()?;
    invitation_dtos.sort_by(|first, second| first.email.cmp(&second.email));
    Ok(invitation_dtos)
}

pub async fn revoke_invitation(
    app_state: &AppState,
    tenant_context: &TenantContext,
    invitation_id: Uuid,
) -> Result<(), InvitationError> {
    authorization_service::require_capability(tenant_context, Permission::MemberInvite)?;
    let revoked = repository::revoke_pending_invitation(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        invitation_id,
    )
    .await?;
    if revoked == 0 {
        return Err(InvitationError::NotFound);
    }
    Ok(())
}

pub async fn revoke_invitations_sent_by_transaction(
    txn: &sea_orm::DatabaseTransaction,
    tenant_id: &TenantId,
    invited_by_user_id: Uuid,
) -> Result<u64, sea_orm::DbErr> {
    repository::revoke_pending_invitations_sent_by_transaction(txn, tenant_id, invited_by_user_id)
        .await
}

pub async fn list_received_invitations(
    app_state: &AppState,
    user_id: UserId,
) -> Result<Vec<ReceivedInvitationDto>, InvitationError> {
    let user = authentication_service::find_user_by_id(app_state, user_id).await?;
    let invitations = repository::list_unexpired_pending_invitations_for_email(
        &app_state.database_connection,
        &authentication_service::normalize_email(&user.email),
        time_utils::now_utc_into(),
    )
    .await?;
    let role_names = role_names_by_id(app_state, &invitations).await?;
    let inviter_names = inviter_names_by_id(app_state, &invitations).await?;
    let tenant_names = crate::tenant::service::find_tenant_names(
        app_state,
        invitations
            .iter()
            .map(|invitation| invitation.tenant_id)
            .collect(),
    )
    .await?;

    let mut received = invitations
        .into_iter()
        .map(|invitation| {
            Ok(ReceivedInvitationDto {
                id: invitation.id,
                tenant_id: invitation.tenant_id,
                tenant_name: lookup(&tenant_names, invitation.tenant_id, "workspace")?,
                role_name: lookup(&role_names, invitation.role_id, "role")?,
                invited_by_name: lookup(&inviter_names, invitation.invited_by_user_id, "inviter")?,
                expires_at: invitation.expires_at,
            })
        })
        .collect::<Result<Vec<_>, InvitationError>>()?;
    received.sort_by(|first, second| first.tenant_name.cmp(&second.tenant_name));
    Ok(received)
}

pub async fn accept_invitation(
    app_state: &AppState,
    user_id: UserId,
    invitation_id: Uuid,
) -> Result<AcceptedInvitationDto, InvitationError> {
    let user = authentication_service::find_user_by_id(app_state, user_id).await?;
    let invitation =
        repository::find_invitation_by_id(&app_state.database_connection, invitation_id)
            .await?
            .ok_or(InvitationError::NotFound)?;
    let addressed_to_caller =
        invitation.email == authentication_service::normalize_email(&user.email);
    if !addressed_to_caller || invitation.status != InvitationStatus::Pending.as_str() {
        return Err(InvitationError::NotFound);
    }
    if effective_status(&invitation) == InvitationStatus::Expired {
        return Err(InvitationError::Expired);
    }

    let tenant_id = TenantId::from_system_unchecked(invitation.tenant_id);
    let already_member =
        membership_service::is_active_member(app_state, &tenant_id, user_id.get_uuid()).await?;

    let txn = app_state.database_connection.begin().await?;
    let accepted =
        repository::accept_pending_invitation_transaction(&txn, invitation.id, user.id).await?;
    if accepted == 0 {
        return Err(InvitationError::NotFound);
    }
    if !already_member {
        membership_service::add_member_transaction(
            &txn,
            &tenant_id,
            user_id,
            invitation.role_id,
            &Actor::human(user_id),
        )
        .await?;
    }
    txn.commit().await?;

    Ok(AcceptedInvitationDto {
        tenant_id: invitation.tenant_id,
    })
}

fn is_valid_email(email: &str) -> bool {
    let Some((local_part, domain)) = email.split_once('@') else {
        return false;
    };
    !local_part.is_empty()
        && !domain.contains('@')
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !email.chars().any(char::is_whitespace)
}

fn effective_status(invitation: &tenant_invitation::Model) -> InvitationStatus {
    let now: DateTimeWithTimeZone = time_utils::now_utc_into();
    if invitation.expires_at <= now {
        return InvitationStatus::Expired;
    }
    InvitationStatus::Pending
}

fn lookup(names: &HashMap<Uuid, String>, id: Uuid, kind: &str) -> Result<String, InvitationError> {
    names.get(&id).cloned().ok_or_else(|| {
        InvitationError::Internal(format!("Invitation points at a missing {kind} {id}"))
    })
}

async fn role_names_by_id(
    app_state: &AppState,
    invitations: &[tenant_invitation::Model],
) -> Result<HashMap<Uuid, String>, InvitationError> {
    Ok(authorization_service::find_roles_by_ids(
        app_state,
        invitations
            .iter()
            .map(|invitation| invitation.role_id)
            .collect(),
    )
    .await?
    .into_iter()
    .map(|role| (role.id, role.name))
    .collect())
}

async fn inviter_names_by_id(
    app_state: &AppState,
    invitations: &[tenant_invitation::Model],
) -> Result<HashMap<Uuid, String>, InvitationError> {
    Ok(authentication_service::find_users_by_ids(
        app_state,
        invitations
            .iter()
            .map(|invitation| invitation.invited_by_user_id)
            .collect(),
    )
    .await?
    .into_iter()
    .map(|user| (user.id, user.full_name))
    .collect())
}
