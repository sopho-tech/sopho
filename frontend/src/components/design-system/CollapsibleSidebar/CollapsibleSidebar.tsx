import {
  ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { NavLink } from "react-router";
import { Icon } from "src/components/design-system/Icon";
import { IconType } from "src/components/design-system/datatypes";
import { HOVER_INTENT_DELAY_MILLISECONDS } from "src/components/design-system/animation";
import {
  CollapsibleSidebarContext,
  useCollapsibleSidebar,
} from "src/components/design-system/CollapsibleSidebar/context";
import styles from "src/components/design-system/CollapsibleSidebar/CollapsibleSidebar.module.css";

const SIDEBAR_ICON_STROKE_WIDTH = 1.75;

export type CollapsibleSidebarProps = {
  isPinned: boolean;
  onPinnedChange: (isPinned: boolean) => void;
  header: ReactNode;
  children: ReactNode;
};

export function CollapsibleSidebar({
  isPinned,
  onPinnedChange,
  header,
  children,
}: CollapsibleSidebarProps) {
  const [isHovered, setIsHovered] = useState(false);
  const [isHeldOpen, setIsHeldOpen] = useState(false);
  const hoverTimeout = useRef<number | null>(null);

  const clearHoverTimeout = useCallback(() => {
    if (hoverTimeout.current !== null) {
      window.clearTimeout(hoverTimeout.current);
      hoverTimeout.current = null;
    }
  }, []);
  useEffect(() => clearHoverTimeout, [clearHoverTimeout]);

  const handleMouseEnter = useCallback(() => {
    clearHoverTimeout();
    hoverTimeout.current = window.setTimeout(
      () => setIsHovered(true),
      HOVER_INTENT_DELAY_MILLISECONDS,
    );
  }, [clearHoverTimeout]);
  const handleMouseLeave = useCallback(() => {
    clearHoverTimeout();
    setIsHovered(false);
  }, [clearHoverTimeout]);

  const isExpanded = isPinned || isHovered || isHeldOpen;
  const contextValue = useMemo(
    () => ({
      isExpanded,
      isPinned,
      setIsPinned: onPinnedChange,
      setIsHeldOpen,
    }),
    [isExpanded, isPinned, onPinnedChange],
  );

  return (
    <CollapsibleSidebarContext value={contextValue}>
      <div className={styles.root} data-expanded={isExpanded}>
        <nav
          className={styles.panel}
          data-expanded={isExpanded}
          onMouseLeave={handleMouseLeave}
        >
          <div className={styles.topBar}>
            <CollapsibleSidebarPinToggle />
          </div>
          <div className={styles.body} onMouseEnter={handleMouseEnter}>
            <div className={styles.header}>{header}</div>
            {children}
          </div>
        </nav>
      </div>
    </CollapsibleSidebarContext>
  );
}

function CollapsibleSidebarPinToggle() {
  const { isPinned, setIsPinned } = useCollapsibleSidebar();
  const label = isPinned ? "Collapse sidebar" : "Keep sidebar open";
  const handleClick = useCallback(
    () => setIsPinned(!isPinned),
    [isPinned, setIsPinned],
  );

  return (
    <button
      type="button"
      className={`${styles.row} ${styles.toggle}`}
      aria-label={label}
      aria-pressed={isPinned}
      onClick={handleClick}
    >
      <span className={styles.leading}>
        <Icon
          type={isPinned ? "panel_left_close" : "panel_left_open"}
          color="current"
          strokeWidth={SIDEBAR_ICON_STROKE_WIDTH}
        />
      </span>
    </button>
  );
}

type CollapsibleSidebarNavigationProps = {
  children: ReactNode;
};

function CollapsibleSidebarNavigation({
  children,
}: CollapsibleSidebarNavigationProps) {
  return <ul className={styles.navigation}>{children}</ul>;
}

type CollapsibleSidebarFooterProps = {
  children: ReactNode;
};

function CollapsibleSidebarFooter({ children }: CollapsibleSidebarFooterProps) {
  return <div className={styles.footer}>{children}</div>;
}

type CollapsibleSidebarLinkProps = {
  to: string;
  icon: IconType;
  label: string;
  isActive: boolean;
};

function CollapsibleSidebarLink({
  to,
  icon,
  label,
  isActive,
}: CollapsibleSidebarLinkProps) {
  return (
    <li>
      <NavLink
        to={to}
        className={styles.row}
        data-active={isActive}
        aria-label={label}
      >
        <span className={styles.leading}>
          <Icon
            type={icon}
            color="current"
            strokeWidth={SIDEBAR_ICON_STROKE_WIDTH}
          />
        </span>
        <span className={styles.label}>{label}</span>
      </NavLink>
    </li>
  );
}

type CollapsibleSidebarHeaderButtonProps = Omit<
  React.ButtonHTMLAttributes<HTMLButtonElement>,
  "children"
> & {
  image: { src: string; alt: string };
  label: string;
  trailingIcon: IconType;
  ref?: React.Ref<HTMLButtonElement>;
};

function CollapsibleSidebarHeaderButton({
  image,
  label,
  trailingIcon,
  ref,
  ...buttonProps
}: CollapsibleSidebarHeaderButtonProps) {
  return (
    <button
      type="button"
      {...buttonProps}
      ref={ref}
      className={`${styles.row} ${styles.headerRow}`}
    >
      <span className={styles.leading}>
        <img src={image.src} alt={image.alt} className={styles.leadingImage} />
      </span>
      <span className={`${styles.label} ${styles.headerLabel}`}>{label}</span>
      <span className={styles.trailing}>
        <Icon
          type={trailingIcon}
          color="current"
          size="sm"
          strokeWidth={SIDEBAR_ICON_STROKE_WIDTH}
        />
      </span>
    </button>
  );
}

CollapsibleSidebar.Navigation = CollapsibleSidebarNavigation;
CollapsibleSidebar.Footer = CollapsibleSidebarFooter;
CollapsibleSidebar.Link = CollapsibleSidebarLink;
CollapsibleSidebar.HeaderButton = CollapsibleSidebarHeaderButton;
