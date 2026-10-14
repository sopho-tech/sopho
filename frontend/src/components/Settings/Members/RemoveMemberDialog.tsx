import { Dialog } from "src/components/Dialog";
import { Button, Flex } from "src/components/design-system";
import { MemberDto } from "src/api/member/queries";

type RemoveMemberDialogProps = {
  member: MemberDto;
  onConfirm: () => void;
  onClose: () => void;
  isPending: boolean;
};

export function RemoveMemberDialog({
  member,
  onConfirm,
  onClose,
  isPending,
}: RemoveMemberDialogProps) {
  return (
    <Dialog
      shouldOpenDialog={true}
      handleOnOpenChange={onClose}
      handleDialogClose={onClose}
      size="sm"
      title={`Remove ${member.full_name}?`}
      description="They lose access to this workspace immediately, along with anything shared with them in it."
      info={
        <Flex gap="sm" justifyContent="flex-end">
          <Button
            type="button"
            label="Cancel"
            onClick={onClose}
            backgroundColor="white"
            size="sm"
            shape="rectangle"
            disabled={isPending}
          />
          <Button
            type="button"
            label="Remove"
            onClick={onConfirm}
            backgroundColor="red"
            size="sm"
            shape="rectangle"
            disabled={isPending}
          />
        </Flex>
      }
    />
  );
}
