import {
  IconColor,
  IconSize,
  IconType,
} from "src/components/design-system/datatypes";
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
  TriangleAlert,
  Hash,
  Type,
  Calendar,
  CheckSquare,
  Key,
  LucideIcon,
  Info,
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
  triangle_alert: TriangleAlert,
  hash: Hash,
  type: Type,
  calendar: Calendar,
  check_square: CheckSquare,
  key: Key,
  info: Info,
};

const sizeMap: Record<IconSize, number> = {
  sm: 14,
  md: 20,
  lg: 24,
};

type IconProps = {
  type: IconType;
  color: IconColor;
  strokeWidth?: number;
  size?: IconSize;
};

export function Icon({ type, color, strokeWidth = 2, size = "md" }: IconProps) {
  const className = styles[color];
  const IconComponent = iconMap[type];
  const iconSize = sizeMap[size];
  return (
    <IconComponent
      size={iconSize}
      className={`${styles.icon} ${className}`}
      strokeWidth={strokeWidth}
    />
  );
}
