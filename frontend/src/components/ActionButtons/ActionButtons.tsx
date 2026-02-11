import styles from "src/components/ActionButtons/ActionButtons.module.css";
import { DropdownMenu, Icon } from "src/components/design-system";
import type { DropdownMenuItem } from "src/components/design-system";

interface ActionButtonsProps {
  connectionId: string;
  onViewClick: (id: string) => void;
  onEditClick: (id: string) => void;
  onDeleteClick: (id: string) => void;
}

export function ActionButtons({
  connectionId,
  onViewClick,
  onEditClick,
  onDeleteClick,
}: ActionButtonsProps) {
  const items: DropdownMenuItem[] = [
    {
      icon: "visibility",
      label: "View",
      onClick: () => onViewClick(connectionId),
    },
    {
      icon: "edit",
      label: "Edit",
      onClick: () => onEditClick(connectionId),
    },
    {
      icon: "delete",
      label: "Delete",
      onClick: () => onDeleteClick(connectionId),
    },
  ];

  return (
    <div onClick={(e) => e.stopPropagation()}>
      <DropdownMenu>
        <DropdownMenu.Trigger>
          <button className={styles.trigger}>
            <Icon type="more_vert" color="grey" size="md" />
          </button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Content align="start" side="bottom">
          {items.map((item, index) => (
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
    </div>
  );
}
