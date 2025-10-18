import "material-symbols";
import NewAssetButtonStyles from "src/components/NewAssetButton/NewAssetButton.module.css";

export enum ButtonType {
  Button = "button",
  Submit = "submit",
  Reset = "reset",
}

export enum ButtonStyle {
  AccentButton = "accent",
  BackButton = "back",
}

export function NewAssetButton({
  buttonText,
  onClick = () => {},
  isLoading = false,
  type = ButtonType.Button,
  style = ButtonStyle.AccentButton,
  className = null,
}: {
  buttonText: string;
  onClick?: (e?: React.MouseEvent) => void;
  isLoading?: boolean;
  type?: ButtonType;
  style?: ButtonStyle;
  className?: string | null;
}) {
  const getButtonStyleClass = () => {
    switch (style) {
      case ButtonStyle.BackButton:
        return NewAssetButtonStyles.back__buttonStyle;
      case ButtonStyle.AccentButton:
      default:
        return NewAssetButtonStyles.buttonStyle;
    }
  };

  return (
    <button
      type={type}
      className={`${className || ""} ${getButtonStyleClass()}`}
      onClick={onClick}
      disabled={isLoading}
    >
      <span className={NewAssetButtonStyles.buttonText}>
        {isLoading ? "Loading..." : buttonText}
      </span>
    </button>
  );
}
