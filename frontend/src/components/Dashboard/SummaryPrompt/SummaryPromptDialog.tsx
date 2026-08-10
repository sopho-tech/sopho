import { useCallback, useEffect, useState } from "react";
import { Dialog } from "src/components/Dialog";
import { Button, Flex, Text, TextArea } from "src/components/design-system";

const MAX_PROMPT_LENGTH = 500;

const PLACEHOLDER = "e.g. Focus on Monday's trends and flag anything unusual";

type SummaryPromptDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onClose: () => void;
  title: string;
  description: string;
  initialPrompt: string | null;
  isSaving: boolean;
  onSave: (userPrompt: string | null) => void;
};

export function SummaryPromptDialog({
  open,
  onOpenChange,
  onClose,
  title,
  description,
  initialPrompt,
  isSaving,
  onSave,
}: SummaryPromptDialogProps) {
  const [value, setValue] = useState(initialPrompt ?? "");

  useEffect(() => {
    if (open) {
      setValue(initialPrompt ?? "");
    }
  }, [open, initialPrompt]);

  const trimmed = value.trim();
  const isTooLong = trimmed.length > MAX_PROMPT_LENGTH;

  const handleChange = useCallback(
    (event: React.ChangeEvent<HTMLTextAreaElement>) => setValue(event.target.value),
    []
  );

  const handleSave = useCallback(() => {
    if (isTooLong) {
      return;
    }
    onSave(trimmed === "" ? null : trimmed);
  }, [isTooLong, trimmed, onSave]);

  return (
    <Dialog
      shouldOpenDialog={open}
      handleOnOpenChange={onOpenChange}
      handleDialogClose={onClose}
      title={title}
      description={description}
      size="sm"
      info={
        <Flex direction="column" gap="md" paddingTop="2xs">
          <Flex direction="column" gap="2xs">
            <TextArea
              value={value}
              onChange={handleChange}
              placeholder={PLACEHOLDER}
              rows={4}
              autoFocus
              disabled={isSaving}
            />
            <Flex direction="row" justifyContent="flex-end">
              <Text as="span" fontSize="xs" color={isTooLong ? "error" : "subtle"}>
                {`${trimmed.length} / ${MAX_PROMPT_LENGTH}`}
              </Text>
            </Flex>
          </Flex>

          <Flex direction="row" justifyContent="flex-end" gap="sm">
            <Button
              label="Cancel"
              shape="rectangle"
              backgroundColor="ghost"
              size="md"
              onClick={onClose}
              disabled={isSaving}
            />
            <Button
              label="Save"
              shape="rectangle"
              backgroundColor="accent"
              size="md"
              onClick={handleSave}
              disabled={isSaving || isTooLong}
            />
          </Flex>
        </Flex>
      }
    />
  );
}
