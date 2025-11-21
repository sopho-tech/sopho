import ExecuteButtonStyles from "src/components/ExecuteButton/ExecuteButton.module.css";
import { Icon } from "src/components/design-system/Icon";

interface ExecuteButtonProps {
  onClick: () => void;
}

export function ExecuteButton({ onClick }: ExecuteButtonProps) {
  return (
    <button className={ExecuteButtonStyles.button} onClick={onClick}>
      <Icon type="play" color="default" />
    </button>
  );
}
