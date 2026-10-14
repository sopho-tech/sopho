import { useCallback, useState } from "react";
import {
  BannerSlim,
  Flex,
  Heading,
  IconButton,
  Select,
  Text,
} from "src/components/design-system";
import {
  MemberDto,
  RoleDto,
  useRemoveMember,
  useRoles,
  useTenantMembers,
  useUpdateMemberRole,
} from "src/api/member/queries";
import { useTenantPermission } from "src/api/tenant/queries";
import { useOidcProviders } from "src/api/auth_api";
import { PERMISSIONS } from "src/constants/permissions";
import { InviteMemberForm } from "src/components/Settings/Members/InviteMemberForm";
import { PendingInvitations } from "src/components/Settings/Members/PendingInvitations";
import { RemoveMemberDialog } from "src/components/Settings/Members/RemoveMemberDialog";

const REMOVE_MEMBER_TOOLTIP = { text: "Remove from workspace" };

function MemberRow({
  member,
  assignableRoles,
  onChangeRole,
  onRemove,
}: {
  member: MemberDto;
  assignableRoles: RoleDto[];
  onChangeRole: (userId: string, roleName: string) => void;
  onRemove: (member: MemberDto) => void;
}) {
  const handleRoleChange = useCallback(
    (roleName: string) => onChangeRole(member.user_id, roleName),
    [member.user_id, onChangeRole],
  );
  const handleRemove = useCallback(() => onRemove(member), [member, onRemove]);

  return (
    <Flex
      direction="row"
      justifyContent="space-between"
      alignItems="center"
      gap="md"
      role="group"
      aria-label={member.email}
    >
      <Flex direction="column">
        <Text fontSize="sm">{member.full_name}</Text>
        <Text fontSize="xs" color="subtle">
          {member.email}
        </Text>
      </Flex>
      <Flex direction="row" alignItems="center" gap="xs">
        {member.capabilities.can_change_role ? (
          <Select value={member.role_name} onValueChange={handleRoleChange}>
            <Select.Trigger />
            <Select.Content>
              {assignableRoles.map((role) => (
                <Select.Item key={role.name} value={role.name}>
                  {role.name}
                </Select.Item>
              ))}
            </Select.Content>
          </Select>
        ) : (
          <Text fontSize="sm" color="subtle">
            {member.role_name}
          </Text>
        )}
        {member.capabilities.can_remove && (
          <IconButton
            type="delete"
            backgroundColor="default"
            iconColor="black"
            iconSize="md"
            onClick={handleRemove}
            tooltip={REMOVE_MEMBER_TOOLTIP}
          />
        )}
      </Flex>
    </Flex>
  );
}

export function Members() {
  const members = useTenantMembers();
  const roles = useRoles();
  const canInvite = useTenantPermission(PERMISSIONS.MEMBER_INVITE);
  const providers = useOidcProviders();
  const updateMemberRole = useUpdateMemberRole();
  const removeMember = useRemoveMember();
  const [memberPendingRemoval, setMemberPendingRemoval] =
    useState<MemberDto | null>(null);

  const { mutate: updateRole } = updateMemberRole;
  const { mutate: remove } = removeMember;

  const handleChangeRole = useCallback(
    (userId: string, roleName: string) => updateRole({ userId, roleName }),
    [updateRole],
  );
  const handleCloseRemoval = useCallback(
    () => setMemberPendingRemoval(null),
    [],
  );
  const handleConfirmRemoval = useCallback(() => {
    if (!memberPendingRemoval) {
      throw new Error("No member is pending removal");
    }
    remove(memberPendingRemoval.user_id, {
      onSuccess: () => setMemberPendingRemoval(null),
    });
  }, [memberPendingRemoval, remove]);

  const failure =
    members.error ??
    roles.error ??
    providers.error ??
    updateMemberRole.error ??
    removeMember.error;

  return (
    <Flex direction="column" gap="xl" overflow="scrollY">
      <Heading accessbilityLevel={1} size="lg">
        Members
      </Heading>
      {failure && <BannerSlim type="error" message={failure.message} />}
      {canInvite && providers.data && providers.data.length === 0 && (
        <BannerSlim
          type="info"
          message="Inviting people needs a single sign-on provider, because invitees join by signing in with it. Configure one with the OIDC_PROVIDERS setting."
        />
      )}
      {canInvite &&
        providers.data &&
        providers.data.length > 0 &&
        roles.data && <InviteMemberForm roles={roles.data} />}
      {members.data && roles.data && (
        <Flex direction="column" gap="sm">
          <Text fontSize="sm" color="darkGrey">
            People in this workspace
          </Text>
          {members.data.map((member) => (
            <MemberRow
              key={member.user_id}
              member={member}
              assignableRoles={roles.data.filter((role) => role.assignable)}
              onChangeRole={handleChangeRole}
              onRemove={setMemberPendingRemoval}
            />
          ))}
        </Flex>
      )}
      {canInvite && <PendingInvitations />}
      {memberPendingRemoval && (
        <RemoveMemberDialog
          member={memberPendingRemoval}
          onConfirm={handleConfirmRemoval}
          onClose={handleCloseRemoval}
          isPending={removeMember.isPending}
        />
      )}
    </Flex>
  );
}
