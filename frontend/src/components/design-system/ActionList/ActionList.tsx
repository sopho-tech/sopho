import styles from "src/components/design-system/ActionList/ActionList.module.css";

type ActionListRootProps = {
  "aria-label": string;
  children: React.ReactNode;
};

export function ActionList({
  "aria-label": ariaLabel,
  children,
}: ActionListRootProps) {
  return (
    <ul aria-label={ariaLabel} className={styles.list}>
      {children}
    </ul>
  );
}

type ActionListItemProps = {
  leading?: React.ReactNode;
  title: string;
  description?: string;
  trailing?: React.ReactNode;
  isSelected?: boolean;
  onClick?: () => void;
};

function ActionListItem({
  leading,
  title,
  description,
  trailing,
  isSelected = false,
  onClick,
}: ActionListItemProps) {
  const content = (
    <>
      {leading}
      <span className={styles.text}>
        <span className={styles.title}>{title}</span>
        {description && (
          <span className={styles.description}>{description}</span>
        )}
      </span>
      {trailing && <span className={styles.trailing}>{trailing}</span>}
    </>
  );
  return (
    <li className={styles.item}>
      {onClick ? (
        <button
          type="button"
          className={`${styles.row} ${styles.rowButton}`}
          aria-current={isSelected ? "true" : undefined}
          onClick={onClick}
        >
          {content}
        </button>
      ) : (
        <div className={styles.row}>{content}</div>
      )}
    </li>
  );
}

ActionList.Item = ActionListItem;
