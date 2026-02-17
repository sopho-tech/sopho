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
  EyeOff,
  Pencil,
  Trash2,
  Play,
  Minus,
  ArrowUp,
  ArrowDown,
  MoreHorizontal,
  MoreVertical,
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
  GripVertical,
  Star,
  Link,
  Save,
  CircleX,
  Search,
  Sparkles,
  RefreshCw,
  Mail,
  Lock,
  User,
  LogOut,
  Layers,
  BarChart,
  Table,
  LayoutDashboard,
  Plug,
} from "lucide-react";

const iconMap: Record<IconType, LucideIcon> = {
  add: Plus,
  home: Home,
  book: BookOpen,
  settings: Settings,
  close: X,
  circle_x: CircleX,
  swap_horiz: ArrowLeftRight,
  swap_vert: ArrowUpDown,
  visibility: Eye,
  visibility_off: EyeOff,
  edit: Pencil,
  delete: Trash2,
  clear: CircleX,
  play: Play,
  remove: Minus,
  arrow_up: ArrowUp,
  arrow_down: ArrowDown,
  more_horiz: MoreHorizontal,
  more_vert: MoreVertical,
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
  grip_vertical: GripVertical,
  star: Star,
  link: Link,
  save: Save,
  search: Search,
  sparkles: Sparkles,
  refresh: RefreshCw,
  email: Mail,
  lock: Lock,
  user: User,
  logout: LogOut,
  layers: Layers,
  bar_chart: BarChart,
  table: Table,
  layout_dashboard: LayoutDashboard,
  plug: Plug,
};

const sizeMap: Record<IconSize, number> = {
  sm: 14,
  md: 20,
  lg: 24,
  "2xl": 450,
};

type IconProps = {
  type: IconType;
  color: IconColor;
  strokeWidth?: number;
  size?: IconSize;
  interactive?: boolean;
};

export function Icon({
  type,
  color,
  strokeWidth = 2.25,
  size = "md",
  interactive = true,
}: IconProps) {
  const className = styles[color];
  const IconComponent = iconMap[type];
  const iconSize = sizeMap[size];
  return (
    <IconComponent
      size={iconSize}
      className={`${styles.icon} ${className}${interactive ? ` ${styles.interactive}` : ""}`}
      strokeWidth={strokeWidth}
    />
  );
}
