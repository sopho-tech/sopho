import styles from "src/components/ActionButtons/ActionButtons.module.css";

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
      <button
        onClick={() => onViewClick(connectionId)}
        title="View"
        className={styles.actionButton}
      >
        <span className={`material-symbols-outlined ${styles["button-icon"]}`}>
          visibility
        </span>
      </button>
      <button
        onClick={() => onEditClick(connectionId)}
        title="Edit"
        className={styles.actionButton}
      >
        <span className={`material-symbols-outlined ${styles["button-icon"]}`}>
          edit
        </span>
      </button>
      <button
        onClick={() => onDeleteClick(connectionId)}
        title="Delete"
        className={styles.actionButton}
      >
        <span className={`material-symbols-outlined ${styles["button-icon"]}`}>
          delete
        </span>
      </button>
    </div>
  );
}
