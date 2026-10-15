use crate::authorization::constants::{AccessLevel, Action, ResourceType, SystemRole, Visibility};
use crate::authorization::dto::PermissionSet;
use crate::authorization::service::{AccessDecision, decide_without_grant, grant_satisfies};
use std::collections::HashSet;

struct DecisionCase {
    role: SystemRole,
    visibility: Visibility,
    is_owner: bool,
    grant: Option<AccessLevel>,
    action: Action,
    expected_allow: bool,
}

fn permissions_for(role: SystemRole) -> PermissionSet {
    PermissionSet::from_permissions(role.permissions().into_iter().collect::<HashSet<_>>())
}

fn allows(case: &DecisionCase) -> bool {
    match decide_without_grant(
        &permissions_for(case.role),
        ResourceType::Canvas,
        case.action,
        case.is_owner,
        case.visibility,
    ) {
        AccessDecision::Allowed => true,
        AccessDecision::MissingCapability | AccessDecision::NotReachable => false,
        AccessDecision::RequiresGrant => grant_satisfies(case.action, case.grant),
    }
}

fn expect(
    role: SystemRole,
    visibility: Visibility,
    is_owner: bool,
    grant: Option<AccessLevel>,
    action: Action,
    expected_allow: bool,
) -> DecisionCase {
    DecisionCase {
        role,
        visibility,
        is_owner,
        grant,
        action,
        expected_allow,
    }
}

#[test]
fn owner_reaches_its_own_resource_whatever_the_visibility() {
    for visibility in [Visibility::Private, Visibility::Tenant] {
        for action in [Action::Read, Action::Write, Action::Delete, Action::Share] {
            let case = expect(SystemRole::Member, visibility, true, None, action, true);
            assert!(
                allows(&case),
                "a member owning a {visibility:?} canvas must be allowed to {action:?}"
            );
        }
    }
}

#[test]
fn a_role_without_the_capability_is_denied_before_reachability_is_considered() {
    for action in [Action::Write, Action::Delete, Action::Share] {
        let decision = decide_without_grant(
            &permissions_for(SystemRole::Viewer),
            ResourceType::Canvas,
            action,
            true,
            Visibility::Tenant,
        );
        assert_eq!(
            decision,
            AccessDecision::MissingCapability,
            "a viewer owning the resource still fails gate 1 for {action:?}"
        );
    }
}

#[test]
fn decision_table_matches_the_specified_model() {
    let cases = vec![
        expect(
            SystemRole::Member,
            Visibility::Private,
            false,
            None,
            Action::Read,
            false,
        ),
        expect(
            SystemRole::Member,
            Visibility::Tenant,
            false,
            None,
            Action::Read,
            true,
        ),
        expect(
            SystemRole::Member,
            Visibility::Tenant,
            false,
            None,
            Action::Write,
            false,
        ),
        expect(
            SystemRole::Member,
            Visibility::Tenant,
            false,
            None,
            Action::Delete,
            false,
        ),
        expect(
            SystemRole::Member,
            Visibility::Private,
            false,
            Some(AccessLevel::Viewer),
            Action::Read,
            true,
        ),
        expect(
            SystemRole::Member,
            Visibility::Private,
            false,
            Some(AccessLevel::Viewer),
            Action::Write,
            false,
        ),
        expect(
            SystemRole::Member,
            Visibility::Private,
            false,
            Some(AccessLevel::Editor),
            Action::Read,
            true,
        ),
        expect(
            SystemRole::Member,
            Visibility::Private,
            false,
            Some(AccessLevel::Editor),
            Action::Write,
            true,
        ),
        expect(
            SystemRole::Member,
            Visibility::Private,
            false,
            Some(AccessLevel::Editor),
            Action::Share,
            false,
        ),
        expect(
            SystemRole::Admin,
            Visibility::Private,
            false,
            None,
            Action::Read,
            true,
        ),
        expect(
            SystemRole::Admin,
            Visibility::Private,
            false,
            None,
            Action::Write,
            true,
        ),
        expect(
            SystemRole::Admin,
            Visibility::Private,
            false,
            None,
            Action::Share,
            true,
        ),
        expect(
            SystemRole::Owner,
            Visibility::Private,
            false,
            None,
            Action::Delete,
            true,
        ),
        expect(
            SystemRole::Viewer,
            Visibility::Tenant,
            false,
            None,
            Action::Read,
            true,
        ),
        expect(
            SystemRole::Viewer,
            Visibility::Private,
            false,
            Some(AccessLevel::Editor),
            Action::Write,
            false,
        ),
    ];

    for case in cases {
        assert_eq!(
            allows(&case),
            case.expected_allow,
            "role={:?} visibility={:?} is_owner={} grant={:?} action={:?}",
            case.role,
            case.visibility,
            case.is_owner,
            case.grant,
            case.action
        );
    }
}
