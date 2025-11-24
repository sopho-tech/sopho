import styles from "src/components/ActionButtons/ActionButtons.module.css";
import { IconButton } from "../design-system";

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
  return (
    <div className={styles.actionButtonsContainer}>
      <IconButton
        type="visibility"
        backgroundColor="transparent"
        iconColor="black"
        onClick={() => onViewClick(connectionId)}
      />
      <IconButton
        type="edit"
        backgroundColor="transparent"
        iconColor="black"
        onClick={() => onEditClick(connectionId)}
      />
      <IconButton
        type="delete"
        backgroundColor="transparent"
        iconColor="black"
        onClick={() => onDeleteClick(connectionId)}
      />
    </div>
  );
}
