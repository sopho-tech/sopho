import { useCallback, useState } from "react";
import { Dialog } from "src/components/Dialog";
import {
  BannerSlim,
  Button,
  Flex,
  Input,
  Select,
  Text,
} from "src/components/design-system";
import { RoleDto } from "src/api/member/queries";
import { InvitationDto, useCreateInvitation } from "src/api/invitation/queries";

type InviteMemberDialogProps = {
  roles: RoleDto[];
  onInvited: (invitation: InvitationDto) => void;
  onClose: () => void;
};

export function InviteMemberDialog({
  roles,
  onInvited,
  onClose,
}: InviteMemberDialogProps) {
  const [email, setEmail] = useState("");
  const [roleName, setRoleName] = useState("");
  const createInvitation = useCreateInvitation();
  const { mutate: invite } = createInvitation;

  const handleEmailChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) =>
      setEmail(event.target.value),
    [],
  );
  const handleInvite = useCallback(
    () => invite({ email, role_name: roleName }, { onSuccess: onInvited }),
    [invite, email, roleName, onInvited],
  );

  return (
    <Dialog
      shouldOpenDialog={true}
      handleOnOpenChange={onClose}
      handleDialogClose={onClose}
      size="sm"
      title="Invite people"
      description="They join this tenant with the role you choose."
      info={
        <Flex direction="column" gap="lg">
          {createInvitation.error && (
            <BannerSlim type="error" message={createInvitation.error.message} />
          )}
          <Flex direction="column" gap="xs">
            <Text fontSize="sm">Email</Text>
            <Input
              value={email}
              onChange={handleEmailChange}
              placeholder="name@company.com"
              type="email"
              aria-label="Email"
              autoFocus
            />
          </Flex>
          <Flex direction="column" gap="xs">
            <Text fontSize="sm">Role</Text>
            <Select value={roleName} onValueChange={setRoleName}>
              <Select.Trigger
                placeholder="Choose a role"
                aria-label="Role"
                fullWidth
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
          </Flex>
          <Flex gap="sm" justifyContent="flex-end">
            <Button
              type="button"
              label="Cancel"
              onClick={onClose}
              backgroundColor="white"
              size="sm"
              shape="rectangle"
              disabled={createInvitation.isPending}
            />
            <Button
              type="button"
              label="Send invitation"
              onClick={handleInvite}
              backgroundColor="accent"
              size="sm"
              shape="rectangle"
              disabled={
                !email.trim() || !roleName || createInvitation.isPending
              }
            />
          </Flex>
        </Flex>
      }
    />
  );
}
