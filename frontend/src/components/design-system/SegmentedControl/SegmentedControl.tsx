import React from "react";
import SegmentedControlStyles from "src/components/design-system/SegmentedControl/SegmentedControl.module.css";
import { IconType } from "src/components/design-system/datatypes";
import { Icon } from "../Icon";
import { ToolTip } from "src/components/design-system/ToolTip";

export type SegmentedControlOption = {
  label?: string;
  value: string;
  leadingIcon?: IconType;
  tooltip?: string | React.ReactNode;
};

export type SegmentedControlProps = {
  options: SegmentedControlOption[];
  value: string;
  onValueChange: (value: string) => void;
  disabled?: boolean;
  size: "sm" | "md" | "lg";
};

export function SegmentedControl({
  options,
  value,
  onValueChange,
  disabled = false,
}: SegmentedControlProps) {
  return (
    <div className={SegmentedControlStyles.root} role="tablist">
      {options.map((option) => {
        const isSelected = option.value === value;
        const iconColor = isSelected ? "accent" : "grey";
        const button = (
          <button
            type="button"
            role="tab"
            aria-selected={isSelected}
            disabled={disabled}
            className={`${SegmentedControlStyles.segment} ${
              isSelected ? SegmentedControlStyles.segmentSelected : ""
            }`}
            onClick={() => !disabled && onValueChange(option.value)}
          >
            {option.leadingIcon && (
              <span className={SegmentedControlStyles.iconContainer}>
                <Icon
                  type={option.leadingIcon}
                  color={iconColor}
                  size="md"
                ></Icon>
              </span>
            )}
            {option.label && (
              <span className={SegmentedControlStyles.label}>
                {option.label}
              </span>
            )}
          </button>
        );

        if (option.tooltip) {
          return (
            <ToolTip
              key={option.value}
              messageElement={option.tooltip}
              tooltipSide="bottom"
            >
              {button}
            </ToolTip>
          );
        }

        return <React.Fragment key={option.value}>{button}</React.Fragment>;
      })}
    </div>
  );
}
