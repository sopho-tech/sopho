import { ReactNode, useMemo } from "react";
import { DropdownMenu, DropdownMenuItem } from "src/components/design-system";
import { ConversationDto } from "src/api/conversational_analytics";
import { useRenameDialog } from "./Sidebar/useRenameDialog";
import { RenameConversationDialog } from "./Sidebar/RenameConversationDialog";
import { useDeleteConversationDialog } from "./Sidebar/useDeleteConversationDialog";
import { DeleteConversationDialog } from "./Sidebar/DeleteConversationDialog";

type ConversationActionsMenuProps = {
  conversation: ConversationDto;
  onOpenChange?: (open: boolean) => void;
  children: ReactNode;
};

export function ConversationActionsMenu({
  conversation,
  onOpenChange,
  children,
}: ConversationActionsMenuProps) {
  const rename = useRenameDialog();
  const deleteDialog = useDeleteConversationDialog(conversation.id);
  const menuItems: DropdownMenuItem[] = useMemo(
    () => [
      {
        icon: "edit",
        label: "Rename",
        onClick: () => rename.openRename(conversation),
      },
      {
        icon: "delete",
        label: "Delete",
        onClick: () => deleteDialog.openDelete(conversation),
      },
    ],
    [conversation, rename.openRename, deleteDialog.openDelete],
  );
  return (
    <>
      <DropdownMenu onOpenChange={onOpenChange}>
        <DropdownMenu.Trigger asChild>{children}</DropdownMenu.Trigger>
        <DropdownMenu.Content align="end" side="bottom">
          {menuItems.map((item, index) => (
            <DropdownMenu.Item
              key={index}
              icon={item.icon}
              onClick={item.onClick}
              disabled={item.disabled}
            >
              {item.label}
            </DropdownMenu.Item>
          ))}
        </DropdownMenu.Content>
      </DropdownMenu>
      {rename.renameTarget && (
        <RenameConversationDialog
          target={rename.renameTarget}
          open={rename.open}
          onOpenChange={rename.handleOpenChange}
          onClose={rename.close}
          onSubmit={rename.submit}
        />
      )}
      {deleteDialog.deleteTarget && (
        <DeleteConversationDialog
          target={deleteDialog.deleteTarget}
          open={deleteDialog.open}
          onOpenChange={deleteDialog.handleOpenChange}
          onClose={deleteDialog.close}
          onConfirm={deleteDialog.confirm}
          isPending={deleteDialog.isPending}
        />
      )}
    </>
  );
}
