import { useCallback } from "react";
import {
  Avatar,
  Badge,
  Box,
  DropdownMenu,
  Flex,
  IconButton,
  Select,
  Table,
  Text,
} from "src/components/design-system";
import { MemberDto, RoleDto } from "src/api/member/queries";
import {
  MEMBER_ACTIONS_LABEL,
  MEMBERS_TABLE_COLUMNS,
} from "src/components/Settings/Members/constants";

type MemberRowProps = {
  member: MemberDto;
  roles: RoleDto[];
  onChangeRole: (userId: string, roleName: string) => void;
  onRemove: (member: MemberDto) => void;
};

function MemberRow({ member, roles, onChangeRole, onRemove }: MemberRowProps) {
  const handleRoleChange = useCallback(
    (roleName: string) => onChangeRole(member.user_id, roleName),
    [member.user_id, onChangeRole],
  );
  const handleRemove = useCallback(() => onRemove(member), [member, onRemove]);

  return (
    <Table.Row aria-label={member.email}>
      <Table.Cell>
        <Avatar name={member.full_name} />
        <Flex direction="column" overflow="hidden">
          <Flex direction="row" alignItems="center" gap="xs">
            <Text fontSize="sm" truncate>
              {member.full_name}
            </Text>
            {member.is_current_user && <Badge variant="subtle">You</Badge>}
            {member.account_status === "DEACTIVATED" && (
              <Badge variant="yellow">Deactivated</Badge>
            )}
          </Flex>
          <Text fontSize="xs" color="subtle" truncate>
            {member.email}
          </Text>
        </Flex>
      </Table.Cell>
      <Table.Cell>
        {member.capabilities.can_change_role ? (
          <Select value={member.role_name} onValueChange={handleRoleChange}>
            <Select.Trigger
              variant="ghost"
              aria-label={`Role of ${member.full_name}`}
            />
            <Select.Content>
              {roles
                .filter((role) => role.assignable)
                .map((role) => (
                  <Select.Item
                    key={role.name}
                    value={role.name}
                    description={role.description}
                  >
                    {role.display_name}
                  </Select.Item>
                ))}
            </Select.Content>
          </Select>
        ) : (
          <Box paddingLeft="xs">
            <Text fontSize="sm" color="darkGrey">
              {member.role_display_name}
            </Text>
          </Box>
        )}
      </Table.Cell>
      <Table.Cell align="end">
        {member.capabilities.can_remove && (
          <DropdownMenu modal={false}>
            <DropdownMenu.Trigger>
              <IconButton
                type="more_horiz"
                backgroundColor="default"
                iconColor="grey"
                iconSize="md"
                aria-label={`${MEMBER_ACTIONS_LABEL} for ${member.full_name}`}
              />
            </DropdownMenu.Trigger>
            <DropdownMenu.Content>
              <DropdownMenu.Item icon="delete" onClick={handleRemove}>
                Remove from tenant
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu>
        )}
      </Table.Cell>
    </Table.Row>
  );
}

type MembersTableProps = {
  members: MemberDto[];
  roles: RoleDto[];
  onChangeRole: (userId: string, roleName: string) => void;
  onRemove: (member: MemberDto) => void;
};

export function MembersTable({
  members,
  roles,
  onChangeRole,
  onRemove,
}: MembersTableProps) {
  return (
    <Table columns={MEMBERS_TABLE_COLUMNS} aria-label="Members">
      <Table.Header>
        <Table.HeaderCell>Member</Table.HeaderCell>
        <Table.HeaderCell>Role</Table.HeaderCell>
        <Table.HeaderCell />
      </Table.Header>
      {members.map((member) => (
        <MemberRow
          key={member.user_id}
          member={member}
          roles={roles}
          onChangeRole={onChangeRole}
          onRemove={onRemove}
        />
      ))}
    </Table>
  );
}
