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
        iconColor="grey"
        onClick={() => onViewClick(connectionId)}
      />
      <IconButton
        type="edit"
        backgroundColor="transparent"
        iconColor="grey"
        onClick={() => onEditClick(connectionId)}
      />
      <IconButton
        type="delete"
        backgroundColor="transparent"
        iconColor="grey"
        onClick={() => onDeleteClick(connectionId)}
      />
    </div>
  );
}
