import { IconButton } from "src/components/design-system/IconButton";

type BackButtonProps = {
  label: string;
  onClick: () => void;
};

export function BackButton({ label, onClick }: BackButtonProps) {
  return (
    <IconButton
      type="chevron_left"
      backgroundColor="transparent"
      iconColor="default"
      size="md"
      iconSize="lg"
      aria-label={label}
      tooltip={{ text: label, direction: "bottom" }}
      onClick={onClick}
    />
  );
}
