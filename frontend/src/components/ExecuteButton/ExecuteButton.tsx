import ExecuteButtonStyles from "src/components/ExecuteButton/ExecuteButton.module.css";

interface ExecuteButtonProps {
  onClick: () => void;
}

export function ExecuteButton({ onClick }: ExecuteButtonProps) {
  return (
    <button className={ExecuteButtonStyles.button} onClick={onClick}>
      <span className="material-symbols-outlined">play_arrow</span>
    </button>
  );
}
