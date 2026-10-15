use crate::authorization::constants::{AccessLevel, ResourceType, SystemRole, Visibility};
use crate::authorization::dto::TenantContext;
use crate::authorization::errors::{AuthorizationError, TenantContextError};
use crate::common::AppState;
use crate::grant::dto::CreateGrantDto;
use crate::tenant::membership::dto::MemberCapabilities;
use crate::tenant::membership::error::MembershipError;
use crate::tenant::membership::service as membership_service;
use crate::tests::common::app_state::init_test_app_state;
use crate::tests::common::resources::insert_canvas;
use crate::tests::common::tenancy::{context_with_role, create_tenant_with_owner, resolve_context};
use std::collections::HashMap;
use uuid::Uuid;

fn user_of(context: &TenantContext) -> Uuid {
    context.user_id().get_uuid()
}

async fn capabilities_seen_by(
    app_state: &AppState,
    viewer: &TenantContext,
) -> HashMap<Uuid, (bool, bool)> {
    membership_service::list_members(app_state, viewer)
        .await
        .unwrap()
        .into_iter()
        .map(|member| {
            let MemberCapabilities {
                can_change_role,
                can_remove,
            } = member.capabilities;
            (member.member.user_id, (can_change_role, can_remove))
        })
        .collect()
}

#[tokio::test]
async fn an_admin_cannot_promote_anyone_to_owner() {
    let app_state = init_test_app_state("membership_admin_promotion").await;
    let fixture = create_tenant_with_owner(&app_state, "promotion").await;
    let admin = context_with_role(&app_state, fixture.tenant_id, SystemRole::Admin, "admin").await;
    let member =
        context_with_role(&app_state, fixture.tenant_id, SystemRole::Member, "member").await;

    assert!(matches!(
        membership_service::update_member_role(
            &app_state,
            &admin,
            user_of(&member),
            SystemRole::Owner.as_str()
        )
        .await,
        Err(MembershipError::RoleTooPowerful)
    ));
    assert!(
        matches!(
            membership_service::update_member_role(
                &app_state,
                &admin,
                user_of(&admin),
                SystemRole::Owner.as_str()
            )
            .await,
            Err(MembershipError::RoleTooPowerful)
        ),
        "not even themselves"
    );

    let demoted = membership_service::update_member_role(
        &app_state,
        &admin,
        user_of(&member),
        SystemRole::Viewer.as_str(),
    )
    .await
    .unwrap();
    assert_eq!(demoted.member.role_name, SystemRole::Viewer.as_str());
}

#[tokio::test]
async fn an_admin_cannot_change_or_remove_an_owner() {
    let app_state = init_test_app_state("membership_admin_versus_owner").await;
    let fixture = create_tenant_with_owner(&app_state, "owner_protection").await;
    let admin = context_with_role(&app_state, fixture.tenant_id, SystemRole::Admin, "admin").await;
    let fellow_admin =
        context_with_role(&app_state, fixture.tenant_id, SystemRole::Admin, "fellow").await;

    assert!(matches!(
        membership_service::update_member_role(
            &app_state,
            &admin,
            fixture.user_id,
            SystemRole::Member.as_str()
        )
        .await,
        Err(MembershipError::MemberTooPowerful)
    ));
    assert!(matches!(
        membership_service::remove_member(&app_state, &admin, fixture.user_id).await,
        Err(MembershipError::MemberTooPowerful)
    ));
    assert!(
        membership_service::remove_member(&app_state, &admin, user_of(&fellow_admin))
            .await
            .is_ok(),
        "an admin manages fellow admins"
    );
}

#[tokio::test]
async fn the_last_owner_cannot_be_demoted_or_removed() {
    let app_state = init_test_app_state("membership_last_owner").await;
    let fixture = create_tenant_with_owner(&app_state, "last_owner").await;
    let owner = resolve_context(&app_state, fixture.tenant_id, fixture.user_id).await;

    assert!(matches!(
        membership_service::update_member_role(
            &app_state,
            &owner,
            fixture.user_id,
            SystemRole::Admin.as_str()
        )
        .await,
        Err(MembershipError::LastOwner)
    ));
    assert!(matches!(
        membership_service::remove_member(&app_state, &owner, fixture.user_id).await,
        Err(MembershipError::LastOwner)
    ));

    let successor = context_with_role(
        &app_state,
        fixture.tenant_id,
        SystemRole::Member,
        "successor",
    )
    .await;
    membership_service::update_member_role(
        &app_state,
        &owner,
        user_of(&successor),
        SystemRole::Owner.as_str(),
    )
    .await
    .unwrap();
    let stepped_down = membership_service::update_member_role(
        &app_state,
        &owner,
        fixture.user_id,
        SystemRole::Admin.as_str(),
    )
    .await
    .unwrap();
    assert_eq!(
        stepped_down.member.role_name,
        SystemRole::Admin.as_str(),
        "a second owner frees the first to step down"
    );
}

#[tokio::test]
async fn removed_member_loses_access_immediately() {
    let app_state = init_test_app_state("membership_removed_member").await;
    let fixture = create_tenant_with_owner(&app_state, "removal").await;
    let owner = resolve_context(&app_state, fixture.tenant_id, fixture.user_id).await;
    let member =
        context_with_role(&app_state, fixture.tenant_id, SystemRole::Member, "member").await;
    let canvas_id = insert_canvas(
        &app_state,
        fixture.tenant_id,
        "private",
        fixture.user_id,
        Visibility::Private,
    )
    .await;
    crate::grant::service::create_grant(
        app_state.clone(),
        &owner,
        CreateGrantDto {
            resource_type: ResourceType::Canvas.as_str().to_string(),
            resource_id: canvas_id,
            principal_user_id: user_of(&member),
            access_level: AccessLevel::Viewer.as_str().to_string(),
        },
    )
    .await
    .unwrap();

    membership_service::remove_member(&app_state, &owner, user_of(&member))
        .await
        .unwrap();

    let resolved = crate::authorization::service::resolve_tenant_context(
        &app_state,
        member.user_id(),
        fixture.tenant_id,
    )
    .await;
    assert!(
        matches!(resolved, Err(TenantContextError::NotAMember)),
        "a removed member must lose the tenant on the next request"
    );
    let sharing = crate::grant::service::get_sharing(
        app_state.clone(),
        &owner,
        ResourceType::Canvas.as_str().to_string(),
        canvas_id,
    )
    .await
    .unwrap();
    assert!(
        sharing.grants.is_empty(),
        "the removed member's grants go with them"
    );
    assert!(matches!(
        membership_service::remove_member(&app_state, &owner, user_of(&member)).await,
        Err(MembershipError::MemberNotFound)
    ));
}

#[tokio::test]
async fn member_capabilities_match_what_the_rules_allow() {
    let app_state = init_test_app_state("membership_capabilities").await;
    let fixture = create_tenant_with_owner(&app_state, "capabilities").await;
    let owner = resolve_context(&app_state, fixture.tenant_id, fixture.user_id).await;
    let admin = context_with_role(&app_state, fixture.tenant_id, SystemRole::Admin, "admin").await;
    let member =
        context_with_role(&app_state, fixture.tenant_id, SystemRole::Member, "member").await;

    let seen_by_admin = capabilities_seen_by(&app_state, &admin).await;
    assert_eq!(seen_by_admin[&fixture.user_id], (false, false));
    assert_eq!(seen_by_admin[&user_of(&admin)], (true, true));
    assert_eq!(seen_by_admin[&user_of(&member)], (true, true));

    let seen_by_member = capabilities_seen_by(&app_state, &member).await;
    assert!(
        seen_by_member
            .values()
            .all(|capabilities| *capabilities == (false, false))
    );
    assert!(matches!(
        membership_service::update_member_role(
            &app_state,
            &member,
            user_of(&admin),
            SystemRole::Viewer.as_str()
        )
        .await,
        Err(MembershipError::Unauthorized(
            AuthorizationError::MissingPermission(_)
        ))
    ));

    let seen_by_owner = capabilities_seen_by(&app_state, &owner).await;
    assert_eq!(
        seen_by_owner[&fixture.user_id],
        (false, false),
        "the last owner cannot be changed, even by themselves"
    );
    assert_eq!(seen_by_owner[&user_of(&admin)], (true, true));

    let co_owner = context_with_role(
        &app_state,
        fixture.tenant_id,
        SystemRole::Member,
        "co_owner",
    )
    .await;
    membership_service::update_member_role(
        &app_state,
        &owner,
        user_of(&co_owner),
        SystemRole::Owner.as_str(),
    )
    .await
    .unwrap();
    let seen_by_admin_with_two_owners = capabilities_seen_by(&app_state, &admin).await;
    assert_eq!(
        seen_by_admin_with_two_owners[&fixture.user_id],
        (false, false),
        "an owner stays out of an admin's reach even when not the last one"
    );
    assert_eq!(
        capabilities_seen_by(&app_state, &owner).await[&fixture.user_id],
        (true, true),
        "with a co-owner, an owner may step down"
    );

    let roles_for_admin: Vec<(String, bool)> = membership_service::list_roles(&app_state, &admin)
        .await
        .unwrap()
        .into_iter()
        .map(|role| (role.name, role.assignable))
        .collect();
    assert_eq!(
        roles_for_admin,
        vec![
            (SystemRole::Owner.as_str().to_string(), false),
            (SystemRole::Admin.as_str().to_string(), true),
            (SystemRole::Member.as_str().to_string(), true),
            (SystemRole::Viewer.as_str().to_string(), true),
        ]
    );
    assert!(
        membership_service::list_roles(&app_state, &owner)
            .await
            .unwrap()
            .iter()
            .all(|role| role.assignable)
    );
}
