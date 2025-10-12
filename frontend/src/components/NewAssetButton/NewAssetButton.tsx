import "material-symbols";
import NewAssetButtonStyles from "src/components/NewAssetButton/NewAssetButton.module.css";

export function NewAssetButton({
  buttonText,
  onClick = () => {},
  isLoading = false,
  type = "button",
  className = null,
}: {
  buttonText: string;
  onClick?: (e?: React.MouseEvent) => void;
  isLoading?: boolean;
  type?: "button" | "submit" | "reset";
  className?: string | null;
}) {
  return (
    <button
      type={type}
      className={`${className || ""} ${NewAssetButtonStyles.buttonStyle}`}
      onClick={onClick}
      disabled={isLoading}
    >
      <span className={NewAssetButtonStyles.buttonText}>
        {isLoading ? "Loading..." : buttonText}
      </span>
    </button>
  );
}
