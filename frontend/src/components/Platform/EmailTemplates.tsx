import { useCallback, useState } from "react";
import {
  ActionList,
  BannerSlim,
  Box,
  Flex,
  Heading,
  HtmlPreview,
  SegmentedControl,
  Text,
  TextPreview,
} from "src/components/design-system";
import {
  EmailTemplatePreviewDto,
  useEmailTemplates,
} from "src/api/platform/queries";
import {
  EMAIL_FORMATS,
  EMAIL_TEMPLATE_LIST_STYLE,
  EMAIL_TEMPLATE_PREVIEW_STYLE,
  HTML_EMAIL_FORMAT,
  TEXT_EMAIL_FORMAT,
} from "src/components/Platform/constants";

type EmailTemplateListItemProps = {
  emailTemplate: EmailTemplatePreviewDto;
  isSelected: boolean;
  onSelect: (kind: string) => void;
};

function EmailTemplateListItem({
  emailTemplate,
  isSelected,
  onSelect,
}: EmailTemplateListItemProps) {
  const handleSelect = useCallback(
    () => onSelect(emailTemplate.kind),
    [emailTemplate.kind, onSelect],
  );

  return (
    <ActionList.Item
      title={emailTemplate.kind_display_name}
      isSelected={isSelected}
      onClick={handleSelect}
    />
  );
}

type EmailTemplatePreviewProps = {
  emailTemplate: EmailTemplatePreviewDto;
};

function EmailTemplatePreview({ emailTemplate }: EmailTemplatePreviewProps) {
  const [selectedFormat, setSelectedFormat] = useState(HTML_EMAIL_FORMAT);

  return (
    <Flex direction="column" gap="md" sx={EMAIL_TEMPLATE_PREVIEW_STYLE}>
      <Flex
        direction="row"
        justifyContent="space-between"
        alignItems="flex-start"
        gap="md"
      >
        <Flex direction="column" gap="2xs">
          <Heading accessibilityLevel={3} size="lg" textColor="black">
            {emailTemplate.subject}
          </Heading>
          <Text fontSize="sm" color="subtle">
            Sent when: {emailTemplate.sent_when}
          </Text>
        </Flex>
        <SegmentedControl
          size="sm"
          value={selectedFormat}
          onValueChange={setSelectedFormat}
          options={EMAIL_FORMATS}
        />
      </Flex>
      {selectedFormat === HTML_EMAIL_FORMAT && (
        <HtmlPreview
          title={`${emailTemplate.kind_display_name} email`}
          html={emailTemplate.html_body}
        />
      )}
      {selectedFormat === TEXT_EMAIL_FORMAT && (
        <TextPreview
          aria-label={`${emailTemplate.kind_display_name} email as plain text`}
          text={emailTemplate.text_body}
        />
      )}
    </Flex>
  );
}

export function EmailTemplates() {
  const emailTemplates = useEmailTemplates();
  const [selectedKind, setSelectedKind] = useState<string | null>(null);

  if (emailTemplates.error) {
    return <BannerSlim type="error" message={emailTemplates.error.message} />;
  }
  if (!emailTemplates.data) {
    return null;
  }

  const selectedEmailTemplate = selectedKind
    ? emailTemplates.data.find(
        (emailTemplate) => emailTemplate.kind === selectedKind,
      )
    : emailTemplates.data[0];
  if (!selectedEmailTemplate) {
    throw new Error("The selected email template is not in the list");
  }

  return (
    <Flex direction="column" gap="lg">
      <Flex direction="column" gap="2xs">
        <Heading accessibilityLevel={2} size="2xl" textColor="black">
          Email templates
        </Heading>
        <Text fontSize="sm" color="subtle">
          Preview every email Sopho sends, filled with sample data. No real user
          data is shown and nothing is sent.
        </Text>
      </Flex>
      <Flex direction="row" alignItems="flex-start" gap="lg">
        <Box sx={EMAIL_TEMPLATE_LIST_STYLE}>
          <ActionList aria-label="Email templates">
            {emailTemplates.data.map((emailTemplate) => (
              <EmailTemplateListItem
                key={emailTemplate.kind}
                emailTemplate={emailTemplate}
                isSelected={emailTemplate.kind === selectedEmailTemplate.kind}
                onSelect={setSelectedKind}
              />
            ))}
          </ActionList>
        </Box>
        <EmailTemplatePreview
          key={selectedEmailTemplate.kind}
          emailTemplate={selectedEmailTemplate}
        />
      </Flex>
    </Flex>
  );
}
