import { useEffect, useRef, useState } from "react";
import { MotionBox } from "src/components/design-system/MotionBox";
import { IconButton } from "src/components/design-system/IconButton";
import { Input } from "src/components/design-system/Input/Input";
import { Duration, EASE } from "src/components/design-system/animation";
import styles from "src/components/design-system/ExpandableSearch/ExpandableSearch.module.css";

export type ExpandableSearchProps = {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  expandedWidth?: number;
  tooltip?: string;
  onExpandedChange?: (expanded: boolean) => void;
};

const COLLAPSED_WIDTH = 32;

export function ExpandableSearch({
  value,
  onChange,
  placeholder = "Search",
  expandedWidth = 260,
  tooltip = "Search",
  onExpandedChange,
}: ExpandableSearchProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isExpanded) {
      inputRef.current?.focus();
    }
  }, [isExpanded]);

  const setExpanded = (expanded: boolean) => {
    setIsExpanded(expanded);
    onExpandedChange?.(expanded);
  };

  const collapse = () => {
    onChange("");
    setExpanded(false);
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Escape") {
      collapse();
    }
  };

  const handleBlur = () => {
    if (!value.trim()) {
      setExpanded(false);
    }
  };

  const transition = {
    duration: Duration.NORMAL,
    ease: EASE,
  };

  return (
    <MotionBox
      className={styles.container}
      animate={{ width: isExpanded ? expandedWidth : COLLAPSED_WIDTH }}
      initial={false}
      transition={transition}
    >
      <MotionBox
        className={styles.inputLayer}
        animate={{ opacity: isExpanded ? 1 : 0 }}
        initial={false}
        transition={transition}
        sx={{ pointerEvents: isExpanded ? "auto" : "none" }}
      >
        <Input
          ref={inputRef}
          value={value}
          onChange={(event) => onChange(event.target.value)}
          onKeyDown={handleKeyDown}
          onBlur={handleBlur}
          placeholder={placeholder}
          leadingIcon="search"
          containerClassName={styles.inputContainer}
          tabIndex={isExpanded ? 0 : -1}
          aria-hidden={!isExpanded}
          aria-label={placeholder}
        />
      </MotionBox>

      <MotionBox
        className={styles.iconLayer}
        animate={{ opacity: isExpanded ? 0 : 1 }}
        initial={false}
        transition={transition}
        sx={{ pointerEvents: isExpanded ? "none" : "auto" }}
      >
        <IconButton
          type="search"
          backgroundColor="transparent"
          iconColor="grey"
          size="sm"
          tooltip={{ text: tooltip }}
          tabIndex={isExpanded ? -1 : 0}
          onClick={() => setExpanded(true)}
        />
      </MotionBox>
    </MotionBox>
  );
}
