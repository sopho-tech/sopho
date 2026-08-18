import { Button, ToolTip } from "src/components/design-system";

type SlashCommandTriggerProps = {
  onClick: () => void;
  disabled: boolean;
};

export function SlashCommandTrigger({
  onClick,
  disabled,
}: SlashCommandTriggerProps) {
  return (
    <ToolTip messageElement="Insert Command" tooltipSide="top">
      <Button
        label="/ Commands"
        shape="pill"
        backgroundColor="ghost"
        size="sm"
        onClick={onClick}
        disabled={disabled}
      />
    </ToolTip>
  );
}
