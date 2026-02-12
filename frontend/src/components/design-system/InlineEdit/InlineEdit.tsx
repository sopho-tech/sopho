import { useState } from "react";
import { Input } from "src/components/design-system/Input";
import { IconButton } from "src/components/design-system/IconButton";
import { Flex } from "src/components/design-system/Flex";
import { Text, type TextColor } from "src/components/design-system/Text/Text";
import {
  Heading,
  type AccessbilityLevel,
} from "src/components/design-system/Heading/Heading";
import { IconSize } from "src/components/design-system/datatypes";
import headingStyles from "src/components/design-system/Heading/Heading.module.css";
import styles from "./InlineEdit.module.css";
import classNames from "classnames";

type InlineEditProps = {
  value: string;
  onSave: (value: string) => void;
  headingLevel?: AccessbilityLevel;
  placeholder?: string;
  clearButtonSize?: IconSize;
  defaultValue?: string;
  textColor?: TextColor;
  className?: string;
};

export function InlineEdit({
  value,
  onSave,
  headingLevel,
  placeholder,
  clearButtonSize,
  defaultValue,
  textColor,
  className,
}: InlineEditProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState(value);

  const handleAccept = () => {
    onSave(editValue);
    setIsEditing(false);
  };

  const handleReject = () => {
    setEditValue(value);
    setIsEditing(false);
  };

  const handleClick = () => {
    if (!isEditing) {
      setEditValue(value);
      setIsEditing(true);
    }
  };

  if (isEditing) {
    return (
      <Flex
        direction="row"
        alignItems="center"
        gap="sm"
        className={classNames(styles.container, className)}
      >
        <div className={styles.inputWrapper}>
          <Input
            value={editValue}
            onChange={(e) => setEditValue(e.target.value)}
            placeholder={placeholder}
            clearButtonSize={clearButtonSize}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleAccept();
              if (e.key === "Escape") handleReject();
            }}
            autoFocus
            className={classNames([
              headingLevel !== undefined
                ? headingStyles[`heading${headingLevel}`]
                : undefined,
              styles["inlineEdit__input"],
            ])}
            style={
              {
                // width: "500px",
              }
            }
          />
        </div>
        <Flex direction="row" gap="xs" className={styles.actions}>
          <IconButton
            type="check"
            backgroundColor="default"
            iconColor="black"
            iconSize="md"
            onClick={handleAccept}
            tooltip={{ text: "Save" }}
            className={styles.iconButtonBordered}
          />
          <IconButton
            type="close"
            backgroundColor="default"
            iconColor="black"
            iconSize="md"
            onClick={handleReject}
            tooltip={{ text: "Cancel" }}
            className={styles.iconButtonBordered}
          />
        </Flex>
      </Flex>
    );
  }

  const displayValue =
    (value?.trim() ?? "") !== "" ? value : (defaultValue ?? "");

  return (
    <div
      className={classNames(styles.viewMode, className)}
      onClick={handleClick}
    >
      {headingLevel !== undefined ? (
        <Heading accessbilityLevel={headingLevel}>{displayValue}</Heading>
      ) : (
        <Text color={textColor}>{displayValue}</Text>
      )}
    </div>
  );
}
