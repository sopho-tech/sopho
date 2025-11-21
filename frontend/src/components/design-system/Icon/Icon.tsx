import { IconColor, IconType } from "src/components/design-system/datatypes";
import styles from "src/components/design-system/Icon/Icon.module.css";
import {
  Plus,
  Home,
  BookOpen,
  Settings,
  X,
  ArrowLeftRight,
  ArrowUpDown,
  Eye,
  Pencil,
  Trash2,
  Play,
  Minus,
  ArrowUp,
  ArrowDown,
  MoreHorizontal,
  ChevronLeft,
  ChevronRight,
  ChevronDown,
  ChevronUp,
  Check,
  LucideIcon,
} from "lucide-react";

const iconMap: Record<IconType, LucideIcon> = {
  add: Plus,
  home: Home,
  book: BookOpen,
  settings: Settings,
  close: X,
  swap_horiz: ArrowLeftRight,
  swap_vert: ArrowUpDown,
  visibility: Eye,
  edit: Pencil,
  delete: Trash2,
  play: Play,
  remove: Minus,
  arrow_up: ArrowUp,
  arrow_down: ArrowDown,
  more_horiz: MoreHorizontal,
  chevron_left: ChevronLeft,
  chevron_right: ChevronRight,
  chevron_down: ChevronDown,
  chevron_up: ChevronUp,
  check: Check,
};

type IconProps = {
  type: IconType;
  color: IconColor;
  strokeWidth?: number;
};

export function Icon({ type, color, strokeWidth = 2.25 }: IconProps) {
  const className = styles[color];
  const IconComponent = iconMap[type];
  return (
    <IconComponent
      size={20}
      className={`${styles.icon} ${className}`}
      strokeWidth={strokeWidth}
    />
  );
}
