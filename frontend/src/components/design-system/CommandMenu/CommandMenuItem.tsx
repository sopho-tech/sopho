import { Command } from "cmdk";
import { Icon } from "../Icon";
import { IconType } from "../datatypes";

type CommandMenuItemProps = {
  value: string;
  label: string;
  iconType?: IconType;
  onSelect?: (value: string) => void;
};

export const CommandMenuItem = ({
  value,
  label,
  iconType,
  onSelect,
}: CommandMenuItemProps) => {
  return (
    <Command.Item value={value} onSelect={onSelect}>
      {iconType && <Icon type={iconType} color="default"></Icon>}
      {label}
    </Command.Item>
  );
};
