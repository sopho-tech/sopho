import { IconColor, IconType } from "src/components/design-system/datatypes";
import styles from "src/components/design-system/Icon/Icon.module.css";

type IconProps = {
  type: IconType;
  color: IconColor;
};

export function Icon({ type, color }: IconProps) {
  const className = styles[color];
  return (
    <span className={`material-symbols-outlined ${className}`}>{type}</span>
  );
}
