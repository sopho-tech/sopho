use crate::authentication::service as authentication_service;
use crate::authorization::constants::Permission;
use crate::authorization::dto::{Actor, TenantContext, TenantId, UserId};
use crate::authorization::service as authorization_service;
use crate::common::AppState;
use crate::common::time_utils;
use crate::common::token_utils;
use crate::email_delivery::constants::EmailStatus;
use crate::email_delivery::dto::{EmailDeliveryDto, EmailMessage};
use crate::email_delivery::service as email_delivery_service;
use crate::email_template::dto::{InvitationEmail, RenderedEmail};
use crate::email_template::service::{EmailTemplate, format_email_date};
use crate::entity::{tenant_invitation, user};
use crate::tenant::invitation::constants::{
    HOURS_IN_HALF_A_DAY, INVITATION_EXPIRED_LABEL, INVITATION_EXPIRES_TODAY_LABEL,
    INVITATION_EXPIRES_TOMORROW_LABEL, INVITATION_EXPIRY_DAYS, INVITATION_LINK_PATH,
    INVITATIONS_UNAVAILABLE_MESSAGE, InvitationDeliveryMethod, InvitationLinkNextStep,
    InvitationStatus,
};
use crate::tenant::invitation::dto::{
    AcceptedInvitationDto, CreateInvitationDto, InvitationDeliveryDto, InvitationDto,
    InvitationLinkDto, InvitationSignUpDto, ReceivedInvitationDto,
};
use crate::tenant::invitation::error::InvitationError;
use crate::tenant::invitation::repository;
use crate::tenant::invitation::repository::{NewInvitation, ReissuedLink};
use crate::tenant::membership::service as membership_service;
use chrono::{DateTime, Utc};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{DatabaseTransaction, TransactionTrait};
use std::collections::HashMap;
use tower_cookies::Cookies;
use uuid::Uuid;

pub fn invitation_delivery_method(app_state: &AppState) -> InvitationDeliveryMethod {
    if email_delivery_service::is_email_enabled(app_state) {
        InvitationDeliveryMethod::Email
    } else if !app_state.config.oidc_providers.is_empty() {
        InvitationDeliveryMethod::SingleSignOn
    } else {
        InvitationDeliveryMethod::Unavailable
    }
}

pub fn get_invitation_delivery(
    app_state: &AppState,
    tenant_context: &TenantContext,
) -> Result<InvitationDeliveryDto, InvitationError> {
    authorization_service::require_capability(tenant_context, Permission::MemberInvite)?;
    let method = invitation_delivery_method(app_state);
    Ok(InvitationDeliveryDto {
        method,
        unavailable_reason: (method == InvitationDeliveryMethod::Unavailable)
            .then(|| INVITATIONS_UNAVAILABLE_MESSAGE.to_string()),
    })
}

pub async fn create_invitation(
    app_state: &AppState,
    tenant_context: &TenantContext,
    payload: CreateInvitationDto,
) -> Result<InvitationDto, InvitationError> {
    authorization_service::require_capability(tenant_context, Permission::MemberInvite)?;
    let delivery_method = invitation_delivery_method(app_state);
    if delivery_method == InvitationDeliveryMethod::Unavailable {
        return Err(InvitationError::InvitationsUnavailable);
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
        && membership_service::is_active_member(app_state, tenant_id, existing_user.id).await?
    {
        return Err(InvitationError::AlreadyMember);
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
    if let Some(tenant_member_limit) = app_state.config.tenant_member_limit {
        let active_member_count =
            membership_service::count_active_members(app_state, tenant_id).await?;
        let pending_invitation_count =
            repository::count_pending_invitations(&app_state.database_connection, tenant_id)
                .await?;
        if active_member_count + pending_invitation_count >= tenant_member_limit {
            return Err(InvitationError::MemberLimitReached {
                limit: tenant_member_limit,
            });
        }
    }

    let inviter =
        authentication_service::find_user_by_id(app_state, tenant_context.user_id()).await?;
    let expires_at: DateTime<Utc> = time_utils::now_plus_days_into(INVITATION_EXPIRY_DAYS);
    let link_token =
        (delivery_method == InvitationDeliveryMethod::Email).then(token_utils::new_link_token);
    let invitation_email = match &link_token {
        Some(link_token) => Some(
            render_invitation_email(
                app_state,
                InvitationEmailDetails {
                    tenant_id,
                    inviter: &inviter,
                    role_name: &role.name,
                    link_token,
                    expires_at,
                },
            )
            .await?,
        ),
        None => None,
    };

    let txn = app_state.database_connection.begin().await?;
    let invitation = repository::insert_invitation(
        &txn,
        tenant_id,
        NewInvitation {
            email: &email,
            role_id: role.id,
            expires_at: expires_at.into(),
            invited_by_user_id: inviter.id,
            token_hash: link_token.as_deref().map(token_utils::hash_token),
        },
    )
    .await?;
    let email_delivery = match invitation_email {
        Some(invitation_email) => {
            let email_id = queue_invitation_email_transaction(
                &txn,
                app_state,
                QueuedInvitationEmail {
                    tenant_id,
                    recipient: &invitation.email,
                    requested_by_user_id: inviter.id,
                    send_before: expires_at,
                    rendered_email: invitation_email,
                },
            )
            .await?;
            repository::set_latest_email(&txn, invitation.id, email_id).await?;
            Some(EmailDeliveryDto {
                status: EmailStatus::Pending,
                last_error: None,
            })
        }
        None => None,
    };
    txn.commit().await?;

    Ok(InvitationDto {
        id: invitation.id,
        email: invitation.email,
        role_display_name: authorization_service::role_display_name(&role.name),
        role_name: role.name,
        status: InvitationStatus::Pending,
        invited_by_name: inviter.full_name,
        expiry_label: expiry_label(invitation.expires_at),
        expires_at: invitation.expires_at,
        email_delivery,
    })
}

pub async fn resend_invitation(
    app_state: &AppState,
    tenant_context: &TenantContext,
    invitation_id: Uuid,
) -> Result<InvitationDto, InvitationError> {
    authorization_service::require_capability(tenant_context, Permission::MemberInvite)?;
    if !email_delivery_service::is_email_enabled(app_state) {
        return Err(InvitationError::ResendNeedsEmail);
    }
    let tenant_id = tenant_context.tenant_id();
    let invitation = repository::find_pending_invitation(
        &app_state.database_connection,
        tenant_id,
        invitation_id,
    )
    .await?
    .ok_or(InvitationError::NotFound)?;
    let inviter = authentication_service::find_user_by_id(
        app_state,
        UserId::from_system_unchecked(invitation.invited_by_user_id),
    )
    .await?;
    let role_name = lookup(
        &role_names_by_id(app_state, std::slice::from_ref(&invitation)).await?,
        invitation.role_id,
        "role",
    )?;
    let link_token = token_utils::new_link_token();
    let expires_at: DateTime<Utc> = time_utils::now_plus_days_into(INVITATION_EXPIRY_DAYS);
    let invitation_email = render_invitation_email(
        app_state,
        InvitationEmailDetails {
            tenant_id,
            inviter: &inviter,
            role_name: &role_name,
            link_token: &link_token,
            expires_at,
        },
    )
    .await?;

    let txn = app_state.database_connection.begin().await?;
    let email_id = queue_invitation_email_transaction(
        &txn,
        app_state,
        QueuedInvitationEmail {
            tenant_id,
            recipient: &invitation.email,
            requested_by_user_id: tenant_context.user_id().get_uuid(),
            send_before: expires_at,
            rendered_email: invitation_email,
        },
    )
    .await?;
    let reissued = repository::reissue_pending_invitation_link(
        &txn,
        invitation.id,
        ReissuedLink {
            token_hash: token_utils::hash_token(&link_token),
            expires_at: expires_at.into(),
            latest_email_id: email_id,
        },
    )
    .await?;
    if reissued == 0 {
        return Err(InvitationError::NotFound);
    }
    if let Some(previous_email_id) = invitation.latest_email_id {
        email_delivery_service::cancel_email_transaction(&txn, previous_email_id).await?;
    }
    txn.commit().await?;

    Ok(InvitationDto {
        id: invitation.id,
        email: invitation.email,
        role_display_name: authorization_service::role_display_name(&role_name),
        role_name,
        status: InvitationStatus::Pending,
        invited_by_name: inviter.full_name,
        expiry_label: expiry_label(expires_at.into()),
        expires_at: expires_at.into(),
        email_delivery: Some(EmailDeliveryDto {
            status: EmailStatus::Pending,
            last_error: None,
        }),
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
    let email_deliveries = email_delivery_service::find_delivery_statuses(
        app_state,
        invitations
            .iter()
            .filter_map(|invitation| invitation.latest_email_id)
            .collect(),
    )
    .await?;

    let mut invitation_dtos = invitations
        .into_iter()
        .map(|invitation| {
            let role_name = lookup(&role_names, invitation.role_id, "role")?;
            Ok(InvitationDto {
                id: invitation.id,
                status: effective_status(&invitation),
                role_display_name: authorization_service::role_display_name(&role_name),
                role_name,
                expiry_label: expiry_label(invitation.expires_at),
                invited_by_name: lookup(&inviter_names, invitation.invited_by_user_id, "inviter")?,
                email_delivery: invitation
                    .latest_email_id
                    .and_then(|email_id| email_deliveries.get(&email_id).cloned()),
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
    let tenant_id = tenant_context.tenant_id();
    let invitation = repository::find_pending_invitation(
        &app_state.database_connection,
        tenant_id,
        invitation_id,
    )
    .await?
    .ok_or(InvitationError::NotFound)?;
    let txn = app_state.database_connection.begin().await?;
    let revoked = repository::revoke_pending_invitation(&txn, tenant_id, invitation.id).await?;
    if revoked == 0 {
        return Err(InvitationError::NotFound);
    }
    if let Some(email_id) = invitation.latest_email_id {
        email_delivery_service::cancel_email_transaction(&txn, email_id).await?;
    }
    txn.commit().await?;
    Ok(())
}

pub async fn revoke_invitations_sent_by_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    invited_by_user_id: Uuid,
) -> Result<u64, sea_orm::DbErr> {
    let pending_invitations =
        repository::find_pending_invitations_sent_by(txn, tenant_id, invited_by_user_id).await?;
    let revoked = repository::revoke_pending_invitations_sent_by_transaction(
        txn,
        tenant_id,
        invited_by_user_id,
    )
    .await?;
    for email_id in pending_invitations
        .iter()
        .filter_map(|invitation| invitation.latest_email_id)
    {
        email_delivery_service::cancel_email_transaction(txn, email_id).await?;
    }
    Ok(revoked)
}

pub async fn revoke_every_invitation_sent_by_transaction(
    txn: &DatabaseTransaction,
    invited_by_user_id: Uuid,
) -> Result<u64, sea_orm::DbErr> {
    let pending_invitations =
        repository::find_pending_invitations_sent_by_user(txn, invited_by_user_id).await?;
    let revoked =
        repository::revoke_pending_invitations_sent_by_user(txn, invited_by_user_id).await?;
    for email_id in pending_invitations
        .iter()
        .filter_map(|invitation| invitation.latest_email_id)
    {
        email_delivery_service::cancel_email_transaction(txn, email_id).await?;
    }
    Ok(revoked)
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
            let role_name = lookup(&role_names, invitation.role_id, "role")?;
            Ok(ReceivedInvitationDto {
                id: invitation.id,
                tenant_id: invitation.tenant_id,
                tenant_name: lookup(&tenant_names, invitation.tenant_id, "tenant")?,
                role_display_name: authorization_service::role_display_name(&role_name),
                role_name,
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
    accept_for_user(
        app_state,
        &user,
        &invitation,
        AcceptanceSource::ReceivedInvitations,
    )
    .await
}

pub async fn lookup_invitation_link(
    app_state: &AppState,
    link_token: &str,
    signed_in_user: Option<UserId>,
) -> Result<InvitationLinkDto, InvitationError> {
    let invitation = find_invitation_by_link_token(app_state, link_token).await?;
    let next_step = match signed_in_user {
        Some(user_id) => {
            let user = authentication_service::find_user_by_id(app_state, user_id).await?;
            if authentication_service::normalize_email(&user.email) == invitation.email {
                InvitationLinkNextStep::Join
            } else {
                InvitationLinkNextStep::WrongAccount
            }
        }
        None => {
            match authentication_service::find_optional_user_by_email(app_state, &invitation.email)
                .await?
            {
                Some(_) => InvitationLinkNextStep::SignIn,
                None => InvitationLinkNextStep::SignUp,
            }
        }
    };
    let tenant_names =
        crate::tenant::service::find_tenant_names(app_state, vec![invitation.tenant_id]).await?;
    let role_names = role_names_by_id(app_state, std::slice::from_ref(&invitation)).await?;
    let inviter_names = inviter_names_by_id(app_state, std::slice::from_ref(&invitation)).await?;
    Ok(InvitationLinkDto {
        tenant_name: lookup(&tenant_names, invitation.tenant_id, "tenant")?,
        role_name: lookup(&role_names, invitation.role_id, "role")?,
        invited_by_name: lookup(&inviter_names, invitation.invited_by_user_id, "inviter")?,
        email: invitation.email,
        expires_at: invitation.expires_at,
        next_step,
        password_sign_up_available: authentication_service::is_password_sign_in_available(
            app_state,
        ),
    })
}

pub async fn sign_up_from_invitation_link(
    app_state: &AppState,
    cookies: &Cookies,
    sign_up: InvitationSignUpDto,
) -> Result<AcceptedInvitationDto, InvitationError> {
    if !authentication_service::is_password_sign_in_available(app_state) {
        return Err(InvitationError::PasswordSignInOff(
            authentication_service::PasswordFlowError::PasswordSignInOff,
        ));
    }
    let invitation = find_invitation_by_link_token(app_state, &sign_up.token).await?;
    if authentication_service::find_optional_user_by_email(app_state, &invitation.email)
        .await?
        .is_some()
    {
        return Err(InvitationError::AccountExists);
    }
    let full_name = sign_up.full_name.trim();
    if full_name.is_empty() {
        return Err(InvitationError::FullNameRequired);
    }
    let normalized_new_password =
        authentication_service::validate_new_password(&sign_up.password, &invitation.email)?;

    let tenant_id = TenantId::from_system_unchecked(invitation.tenant_id);
    let txn = app_state.database_connection.begin().await?;
    let user = authentication_service::create_password_user_transaction(
        &txn,
        &invitation.email,
        full_name,
        &normalized_new_password,
    )
    .await?;
    let accepted =
        repository::accept_pending_invitation_transaction(&txn, invitation.id, user.id).await?;
    if accepted == 0 {
        return Err(InvitationError::LinkInvalid);
    }
    let new_user_id = UserId::from_system_unchecked(user.id);
    membership_service::add_member_transaction(
        &txn,
        &tenant_id,
        new_user_id,
        invitation.role_id,
        &Actor::human(new_user_id),
    )
    .await?;
    txn.commit().await?;

    authentication_service::start_session_for_user(app_state, cookies, user.id).await?;
    Ok(AcceptedInvitationDto {
        tenant_id: invitation.tenant_id,
    })
}

pub async fn accept_invitation_link(
    app_state: &AppState,
    user_id: UserId,
    link_token: &str,
) -> Result<AcceptedInvitationDto, InvitationError> {
    let invitation = find_invitation_by_link_token(app_state, link_token).await?;
    let user = authentication_service::find_user_by_id(app_state, user_id).await?;
    if authentication_service::normalize_email(&user.email) != invitation.email {
        return Err(InvitationError::InvitationForAnotherAccount);
    }
    accept_for_user(app_state, &user, &invitation, AcceptanceSource::EmailedLink).await
}

enum AcceptanceSource {
    ReceivedInvitations,
    EmailedLink,
}

async fn accept_for_user(
    app_state: &AppState,
    user: &user::Model,
    invitation: &tenant_invitation::Model,
    acceptance_source: AcceptanceSource,
) -> Result<AcceptedInvitationDto, InvitationError> {
    let user_id = UserId::from_system_unchecked(user.id);
    let tenant_id = TenantId::from_system_unchecked(invitation.tenant_id);
    let already_member =
        membership_service::is_active_member(app_state, &tenant_id, user.id).await?;

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
    if let AcceptanceSource::EmailedLink = acceptance_source {
        authentication_service::mark_email_verified_transaction(&txn, user.id).await?;
    }
    txn.commit().await?;

    Ok(AcceptedInvitationDto {
        tenant_id: invitation.tenant_id,
    })
}

async fn find_invitation_by_link_token(
    app_state: &AppState,
    link_token: &str,
) -> Result<tenant_invitation::Model, InvitationError> {
    let invitation = repository::find_pending_invitation_by_token_hash(
        &app_state.database_connection,
        &token_utils::hash_token(link_token),
    )
    .await?
    .ok_or(InvitationError::LinkInvalid)?;
    if effective_status(&invitation) == InvitationStatus::Expired {
        return Err(InvitationError::LinkInvalid);
    }
    Ok(invitation)
}

struct InvitationEmailDetails<'a> {
    tenant_id: &'a TenantId,
    inviter: &'a user::Model,
    role_name: &'a str,
    link_token: &'a str,
    expires_at: DateTime<Utc>,
}

struct QueuedInvitationEmail<'a> {
    tenant_id: &'a TenantId,
    recipient: &'a str,
    requested_by_user_id: Uuid,
    send_before: DateTime<Utc>,
    rendered_email: RenderedEmail,
}

async fn render_invitation_email(
    app_state: &AppState,
    details: InvitationEmailDetails<'_>,
) -> Result<RenderedEmail, InvitationError> {
    let tenant_uuid = details.tenant_id.get_uuid();
    let tenant_names =
        crate::tenant::service::find_tenant_names(app_state, vec![tenant_uuid]).await?;
    Ok(InvitationEmail {
        inviter_name: details.inviter.full_name.clone(),
        tenant_name: lookup(&tenant_names, tenant_uuid, "tenant")?,
        role_name: authorization_service::role_display_name(details.role_name),
        invitation_link: email_delivery_service::public_link(
            app_state,
            &format!("{INVITATION_LINK_PATH}{}", details.link_token),
        )?,
        expires_on: format_email_date(details.expires_at),
    }
    .render()?)
}

async fn queue_invitation_email_transaction(
    txn: &DatabaseTransaction,
    app_state: &AppState,
    queued_email: QueuedInvitationEmail<'_>,
) -> Result<Uuid, InvitationError> {
    Ok(email_delivery_service::enqueue_email_transaction(
        txn,
        app_state,
        EmailMessage {
            kind: queued_email.rendered_email.kind.to_string(),
            recipient: queued_email.recipient.to_string(),
            subject: queued_email.rendered_email.subject,
            text_body: queued_email.rendered_email.text_body,
            html_body: queued_email.rendered_email.html_body,
            send_before: queued_email.send_before,
            tenant_id: Some(queued_email.tenant_id.get_uuid()),
            requested_by_user_id: Some(queued_email.requested_by_user_id),
        },
    )
    .await?)
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

fn expiry_label(expires_at: DateTimeWithTimeZone) -> String {
    let now: DateTimeWithTimeZone = time_utils::now_utc_into();
    if expires_at <= now {
        return INVITATION_EXPIRED_LABEL.to_string();
    }
    let rounded_days_left =
        (expires_at - now + chrono::Duration::hours(HOURS_IN_HALF_A_DAY)).num_days();
    match rounded_days_left {
        0 => INVITATION_EXPIRES_TODAY_LABEL.to_string(),
        1 => INVITATION_EXPIRES_TOMORROW_LABEL.to_string(),
        days => format!("Expires in {days} days"),
    }
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
