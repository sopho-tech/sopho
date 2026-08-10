import { useCallback, useState } from "react";
import {
  Button,
  Flex,
  HoverCard,
  SplitIconButton,
  Text,
} from "src/components/design-system";
import type { SplitIconButtonSize } from "src/components/design-system";
import { IconType } from "src/components/design-system/datatypes";
import { SummaryPromptDialog } from "src/components/Dashboard/SummaryPrompt/SummaryPromptDialog";

type SummaryPromptControlProps = {
  iconType: IconType;
  actionLabel: string;
  dialogTitle: string;
  dialogDescription: string;
  userPrompt: string | null;
  busy: boolean;
  isSaving: boolean;
  size?: SplitIconButtonSize;
  onGenerate: () => void;
  onSavePrompt: (userPrompt: string | null) => void;
};

export function SummaryPromptControl({
  iconType,
  actionLabel,
  dialogTitle,
  dialogDescription,
  userPrompt,
  busy,
  isSaving,
  size = "md",
  onGenerate,
  onSavePrompt,
}: SummaryPromptControlProps) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  const handleClose = useCallback(() => setIsDialogOpen(false), []);
  const handleOpen = useCallback(() => setIsDialogOpen(true), []);

  const handleSave = useCallback(
    (prompt: string | null) => {
      onSavePrompt(prompt);
      setIsDialogOpen(false);
    },
    [onSavePrompt]
  );

  return (
    <>
      <HoverCard>
        <HoverCard.Trigger>
          <SplitIconButton
            type={iconType}
            iconColor="grey"
            size={size}
            onClick={onGenerate}
            busy={busy}
            primaryLabel={actionLabel}
            onSecondaryClick={handleOpen}
            secondaryLabel={`${actionLabel} — edit prompt`}
          />
        </HoverCard.Trigger>
        <HoverCard.Content>
          <Flex direction="column" gap="2xs">
            <Text as="span" fontSize="sm">
              {actionLabel}
            </Text>
            <Text as="span" fontSize="xs" color="subtle">
              Your prompt
            </Text>
            <Text as="p" fontSize="sm">
              {userPrompt ?? "No custom prompt — using the default style."}
            </Text>
            <Flex direction="row" justifyContent="flex-end">
              <Button
                label="Edit"
                shape="rectangle"
                backgroundColor="ghost"
                size="sm"
                onClick={handleOpen}
              />
            </Flex>
          </Flex>
        </HoverCard.Content>
      </HoverCard>

      <SummaryPromptDialog
        open={isDialogOpen}
        onOpenChange={setIsDialogOpen}
        onClose={handleClose}
        title={dialogTitle}
        description={dialogDescription}
        initialPrompt={userPrompt}
        isSaving={isSaving}
        onSave={handleSave}
      />
    </>
  );
}
