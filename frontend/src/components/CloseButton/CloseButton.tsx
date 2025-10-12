import CloseButtonStyles from "src/components/CloseButton/CloseButton.module.css";

export function CloseButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className={CloseButtonStyles.closeButton}
      aria-label="Close"
    >
      <span className="material-symbols-rounded">close</span>
    </button>
  );
}
