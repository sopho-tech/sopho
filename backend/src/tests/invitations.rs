use crate::authorization::constants::SystemRole;
use crate::authorization::dto::{TenantContext, UserId};
use crate::authorization::errors::AuthorizationError;
use crate::common::AppState;
use crate::common::time_utils;
use crate::entity;
use crate::tenant::invitation::constants::InvitationStatus;
use crate::tenant::invitation::dto::CreateInvitationDto;
use crate::tenant::invitation::error::InvitationError;
use crate::tenant::invitation::service as invitation_service;
use crate::tests::common::app_state::{
    init_test_app_state, init_test_app_state_with_single_sign_on,
};
use crate::tests::common::tenancy::{
    context_with_role, create_tenant_with_owner, insert_user, resolve_context,
};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;

async fn email_of(app_state: &AppState, user_id: Uuid) -> String {
    entity::user::Entity::find_by_id(user_id)
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap()
        .email
}

fn invite(email: &str, role: SystemRole) -> CreateInvitationDto {
    CreateInvitationDto {
        email: email.to_string(),
        role_name: role.as_str().to_string(),
    }
}

async fn expire(app_state: &AppState, invitation_id: Uuid) {
    let invitation = entity::tenant_invitation::Entity::find_by_id(invitation_id)
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap();
    let mut expiring: entity::tenant_invitation::ActiveModel = invitation.into();
    expiring.expires_at = Set(time_utils::now_plus_days_into(-1));
    expiring
        .update(&app_state.database_connection)
        .await
        .unwrap();
}

async fn owner_context(app_state: &AppState, tenant_name: &str) -> (Uuid, TenantContext) {
    let fixture = create_tenant_with_owner(app_state, tenant_name).await;
    let owner = resolve_context(app_state, fixture.tenant_id, fixture.user_id).await;
    (fixture.tenant_id, owner)
}

#[tokio::test]
async fn an_invitation_is_accepted_into_the_invited_role() {
    let app_state = init_test_app_state_with_single_sign_on("invitation_accept_role").await;
    let (tenant_id, owner) = owner_context(&app_state, "acme").await;
    let invitee = insert_user(&app_state, "invitee").await;
    let invitee_email = email_of(&app_state, invitee).await;

    let invitation = invitation_service::create_invitation(
        &app_state,
        &owner,
        invite(&invitee_email, SystemRole::Viewer),
    )
    .await
    .unwrap();
    assert_eq!(invitation.status, InvitationStatus::Pending);

    let accepted = invitation_service::accept_invitation(
        &app_state,
        UserId::from_system_unchecked(invitee),
        invitation.id,
    )
    .await
    .unwrap();
    assert_eq!(accepted.tenant_id, tenant_id);

    let memberships = crate::tenant::service::list_tenants_for_user(
        &app_state,
        UserId::from_system_unchecked(invitee),
    )
    .await
    .unwrap();
    assert_eq!(memberships.len(), 1);
    assert_eq!(memberships[0].tenant_id, tenant_id);
    assert_eq!(memberships[0].role_name, SystemRole::Viewer.as_str());
    assert!(
        invitation_service::list_tenant_invitations(&app_state, &owner)
            .await
            .unwrap()
            .is_empty(),
        "an accepted invitation is no longer pending"
    );
}

#[tokio::test]
async fn only_the_invited_email_can_accept() {
    let app_state = init_test_app_state_with_single_sign_on("invitation_addressee").await;
    let (_, owner) = owner_context(&app_state, "acme").await;
    let invitee = insert_user(&app_state, "invitee").await;
    let stranger = insert_user(&app_state, "stranger").await;
    let invitee_email = email_of(&app_state, invitee).await;

    let invitation = invitation_service::create_invitation(
        &app_state,
        &owner,
        invite(&invitee_email.to_uppercase(), SystemRole::Member),
    )
    .await
    .unwrap();

    assert!(matches!(
        invitation_service::accept_invitation(
            &app_state,
            UserId::from_system_unchecked(stranger),
            invitation.id
        )
        .await,
        Err(InvitationError::NotFound)
    ));
    assert!(
        invitation_service::accept_invitation(
            &app_state,
            UserId::from_system_unchecked(invitee),
            invitation.id
        )
        .await
        .is_ok(),
        "addresses compare case-insensitively"
    );
}

#[tokio::test]
async fn expired_and_revoked_invitations_cannot_be_accepted() {
    let app_state = init_test_app_state_with_single_sign_on("invitation_expired_revoked").await;
    let (_, owner) = owner_context(&app_state, "acme").await;
    let invitee = insert_user(&app_state, "invitee").await;
    let invitee_email = email_of(&app_state, invitee).await;

    let revoked = invitation_service::create_invitation(
        &app_state,
        &owner,
        invite(&invitee_email, SystemRole::Member),
    )
    .await
    .unwrap();
    invitation_service::revoke_invitation(&app_state, &owner, revoked.id)
        .await
        .unwrap();
    assert!(matches!(
        invitation_service::accept_invitation(
            &app_state,
            UserId::from_system_unchecked(invitee),
            revoked.id
        )
        .await,
        Err(InvitationError::NotFound)
    ));
    assert!(matches!(
        invitation_service::revoke_invitation(&app_state, &owner, revoked.id).await,
        Err(InvitationError::NotFound)
    ));

    let expiring = invitation_service::create_invitation(
        &app_state,
        &owner,
        invite(&invitee_email, SystemRole::Member),
    )
    .await
    .unwrap();
    expire(&app_state, expiring.id).await;

    let listed = invitation_service::list_tenant_invitations(&app_state, &owner)
        .await
        .unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].status, InvitationStatus::Expired);
    assert!(matches!(
        invitation_service::accept_invitation(
            &app_state,
            UserId::from_system_unchecked(invitee),
            expiring.id
        )
        .await,
        Err(InvitationError::Expired)
    ));
    assert!(
        invitation_service::list_received_invitations(
            &app_state,
            UserId::from_system_unchecked(invitee)
        )
        .await
        .unwrap()
        .is_empty()
    );
}

#[tokio::test]
async fn an_invitation_cannot_be_accepted_twice() {
    let app_state = init_test_app_state_with_single_sign_on("invitation_single_use").await;
    let (_, owner) = owner_context(&app_state, "acme").await;
    let invitee = insert_user(&app_state, "invitee").await;
    let invitee_email = email_of(&app_state, invitee).await;
    let invitation = invitation_service::create_invitation(
        &app_state,
        &owner,
        invite(&invitee_email, SystemRole::Member),
    )
    .await
    .unwrap();

    invitation_service::accept_invitation(
        &app_state,
        UserId::from_system_unchecked(invitee),
        invitation.id,
    )
    .await
    .unwrap();
    assert!(matches!(
        invitation_service::accept_invitation(
            &app_state,
            UserId::from_system_unchecked(invitee),
            invitation.id
        )
        .await,
        Err(InvitationError::NotFound)
    ));
}

#[tokio::test]
async fn inviting_a_member_or_a_pending_email_conflicts() {
    let app_state = init_test_app_state_with_single_sign_on("invitation_conflicts").await;
    let (tenant_id, owner) = owner_context(&app_state, "acme").await;
    let member = context_with_role(&app_state, tenant_id, SystemRole::Member, "member").await;
    let member_email = email_of(&app_state, member.user_id().get_uuid()).await;
    let newcomer_email = "newcomer@example.test";

    assert!(matches!(
        invitation_service::create_invitation(
            &app_state,
            &owner,
            invite(&member_email, SystemRole::Viewer)
        )
        .await,
        Err(InvitationError::AlreadyMember)
    ));

    let first = invitation_service::create_invitation(
        &app_state,
        &owner,
        invite(newcomer_email, SystemRole::Member),
    )
    .await
    .unwrap();
    assert!(matches!(
        invitation_service::create_invitation(
            &app_state,
            &owner,
            invite(" Newcomer@Example.test ", SystemRole::Member)
        )
        .await,
        Err(InvitationError::AlreadyInvited)
    ));

    invitation_service::revoke_invitation(&app_state, &owner, first.id)
        .await
        .unwrap();
    let second = invitation_service::create_invitation(
        &app_state,
        &owner,
        invite(newcomer_email, SystemRole::Member),
    )
    .await
    .unwrap();
    expire(&app_state, second.id).await;
    assert!(
        invitation_service::create_invitation(
            &app_state,
            &owner,
            invite(newcomer_email, SystemRole::Member)
        )
        .await
        .is_ok(),
        "an expired invitation does not block a fresh one"
    );
}

#[tokio::test]
async fn nobody_invites_into_a_role_more_powerful_than_their_own() {
    let app_state = init_test_app_state_with_single_sign_on("invitation_role_ceiling").await;
    let (tenant_id, _) = owner_context(&app_state, "acme").await;
    let admin = context_with_role(&app_state, tenant_id, SystemRole::Admin, "admin").await;
    let member = context_with_role(&app_state, tenant_id, SystemRole::Member, "member").await;

    assert!(matches!(
        invitation_service::create_invitation(
            &app_state,
            &admin,
            invite("future.owner@example.test", SystemRole::Owner)
        )
        .await,
        Err(InvitationError::RoleTooPowerful)
    ));
    assert!(
        invitation_service::create_invitation(
            &app_state,
            &admin,
            invite("future.admin@example.test", SystemRole::Admin)
        )
        .await
        .is_ok()
    );
    assert!(matches!(
        invitation_service::create_invitation(
            &app_state,
            &member,
            invite("friend@example.test", SystemRole::Viewer)
        )
        .await,
        Err(InvitationError::Unauthorized(
            AuthorizationError::MissingPermission(_)
        ))
    ));
    assert!(matches!(
        invitation_service::create_invitation(
            &app_state,
            &admin,
            CreateInvitationDto {
                email: "someone@example.test".to_string(),
                role_name: "SUPERUSER".to_string(),
            }
        )
        .await,
        Err(InvitationError::UnknownRole)
    ));
    assert!(matches!(
        invitation_service::create_invitation(
            &app_state,
            &admin,
            invite("not-an-email", SystemRole::Viewer)
        )
        .await,
        Err(InvitationError::InvalidEmail)
    ));
}

#[tokio::test]
async fn received_invitations_list_only_what_the_caller_can_accept() {
    let app_state = init_test_app_state_with_single_sign_on("invitation_received").await;
    let (first_tenant_id, first_owner) = owner_context(&app_state, "first").await;
    let (_, second_owner) = owner_context(&app_state, "second").await;
    let (_, third_owner) = owner_context(&app_state, "third").await;
    let invitee = insert_user(&app_state, "invitee").await;
    let invitee_email = email_of(&app_state, invitee).await;

    let pending = invitation_service::create_invitation(
        &app_state,
        &first_owner,
        invite(&invitee_email, SystemRole::Member),
    )
    .await
    .unwrap();
    let revoked = invitation_service::create_invitation(
        &app_state,
        &second_owner,
        invite(&invitee_email, SystemRole::Member),
    )
    .await
    .unwrap();
    invitation_service::revoke_invitation(&app_state, &second_owner, revoked.id)
        .await
        .unwrap();
    let expired = invitation_service::create_invitation(
        &app_state,
        &third_owner,
        invite(&invitee_email, SystemRole::Member),
    )
    .await
    .unwrap();
    expire(&app_state, expired.id).await;
    invitation_service::create_invitation(
        &app_state,
        &first_owner,
        invite("someone.else@example.test", SystemRole::Member),
    )
    .await
    .unwrap();

    let received = invitation_service::list_received_invitations(
        &app_state,
        UserId::from_system_unchecked(invitee),
    )
    .await
    .unwrap();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].id, pending.id);
    assert_eq!(received[0].tenant_id, first_tenant_id);
    assert_eq!(received[0].tenant_name, "first");
    assert_eq!(received[0].role_name, SystemRole::Member.as_str());
    assert_eq!(received[0].invited_by_name, "first");
}

#[tokio::test]
async fn inviting_requires_a_single_sign_on_provider() {
    let app_state = init_test_app_state("invitation_needs_single_sign_on").await;
    let (_, owner) = owner_context(&app_state, "acme").await;

    assert!(matches!(
        invitation_service::create_invitation(
            &app_state,
            &owner,
            invite("newcomer@example.test", SystemRole::Member)
        )
        .await,
        Err(InvitationError::SingleSignOnRequired)
    ));
    assert!(
        invitation_service::list_tenant_invitations(&app_state, &owner)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn removing_a_member_revokes_the_invitations_they_sent() {
    let app_state = init_test_app_state_with_single_sign_on("invitation_sender_removed").await;
    let (tenant_id, owner) = owner_context(&app_state, "acme").await;
    let admin = context_with_role(&app_state, tenant_id, SystemRole::Admin, "admin").await;
    let accomplice = insert_user(&app_state, "accomplice").await;
    let accomplice_email = email_of(&app_state, accomplice).await;

    let backdoor = invitation_service::create_invitation(
        &app_state,
        &admin,
        invite(&accomplice_email, SystemRole::Admin),
    )
    .await
    .unwrap();
    let unrelated = invitation_service::create_invitation(
        &app_state,
        &owner,
        invite("friend@example.test", SystemRole::Viewer),
    )
    .await
    .unwrap();

    crate::tenant::membership::service::remove_member(
        &app_state,
        &owner,
        admin.user_id().get_uuid(),
    )
    .await
    .unwrap();

    assert!(
        matches!(
            invitation_service::accept_invitation(
                &app_state,
                UserId::from_system_unchecked(accomplice),
                backdoor.id
            )
            .await,
            Err(InvitationError::NotFound)
        ),
        "a removed member's invitations cannot bring anyone back"
    );
    let remaining: Vec<Uuid> = invitation_service::list_tenant_invitations(&app_state, &owner)
        .await
        .unwrap()
        .into_iter()
        .map(|invitation| invitation.id)
        .collect();
    assert_eq!(
        remaining,
        vec![unrelated.id],
        "invitations sent by others stay pending"
    );
}

#[tokio::test]
async fn accepting_while_already_a_member_keeps_the_existing_role() {
    let app_state = init_test_app_state_with_single_sign_on("invitation_existing_member").await;
    let (tenant_id, owner) = owner_context(&app_state, "acme").await;
    let invitee = insert_user(&app_state, "invitee").await;
    let invitee_email = email_of(&app_state, invitee).await;
    let invitation = invitation_service::create_invitation(
        &app_state,
        &owner,
        invite(&invitee_email, SystemRole::Admin),
    )
    .await
    .unwrap();
    crate::tenant::membership::service::add_member_with_role_name(
        &app_state,
        &crate::authorization::dto::TenantId::from_system_unchecked(tenant_id),
        UserId::from_system_unchecked(invitee),
        SystemRole::Viewer.as_str(),
        &crate::authorization::dto::Actor::system(),
    )
    .await
    .unwrap();

    let accepted = invitation_service::accept_invitation(
        &app_state,
        UserId::from_system_unchecked(invitee),
        invitation.id,
    )
    .await
    .unwrap();

    assert_eq!(accepted.tenant_id, tenant_id);
    let memberships = crate::tenant::service::list_tenants_for_user(
        &app_state,
        UserId::from_system_unchecked(invitee),
    )
    .await
    .unwrap();
    assert_eq!(memberships.len(), 1);
    assert_eq!(
        memberships[0].role_name,
        SystemRole::Viewer.as_str(),
        "an invitation never silently changes an existing member's role"
    );
    assert!(
        invitation_service::list_tenant_invitations(&app_state, &owner)
            .await
            .unwrap()
            .is_empty()
    );
}
