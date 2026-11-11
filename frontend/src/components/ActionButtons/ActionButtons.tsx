import { useCallback, useMemo } from "react";
import styles from "src/components/ActionButtons/ActionButtons.module.css";
import { DropdownMenu, Icon } from "src/components/design-system";
import type { DropdownMenuItem } from "src/components/design-system";
import type { ResourceCapabilities } from "src/api/dto";

interface ActionButtonsProps {
  connectionId: string;
  capabilities: ResourceCapabilities;
  onViewClick: (id: string) => void;
  onEditClick: (id: string) => void;
  onDeleteClick: (id: string) => void;
}

export function ActionButtons({
  connectionId,
  capabilities,
  onViewClick,
  onEditClick,
  onDeleteClick,
}: ActionButtonsProps) {
  const handleViewClick = useCallback(
    () => onViewClick(connectionId),
    [connectionId, onViewClick],
  );
  const handleEditClick = useCallback(
    () => onEditClick(connectionId),
    [connectionId, onEditClick],
  );
  const handleDeleteClick = useCallback(
    () => onDeleteClick(connectionId),
    [connectionId, onDeleteClick],
  );

  const handleStopPropagation = useCallback(
    (e: React.MouseEvent) => e.stopPropagation(),
    [],
  );

  const items: DropdownMenuItem[] = useMemo(
    () => [
      { icon: "visibility", label: "View", onClick: handleViewClick },
      ...(capabilities.can_edit
        ? [{ icon: "edit" as const, label: "Edit", onClick: handleEditClick }]
        : []),
      ...(capabilities.can_delete
        ? [
            {
              icon: "delete" as const,
              label: "Delete",
              onClick: handleDeleteClick,
            },
          ]
        : []),
    ],
    [capabilities, handleViewClick, handleEditClick, handleDeleteClick],
  );

  return (
    <div onClick={handleStopPropagation}>
      <DropdownMenu>
        <DropdownMenu.Trigger>
          <button className={styles.trigger} aria-label="Connection actions">
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
