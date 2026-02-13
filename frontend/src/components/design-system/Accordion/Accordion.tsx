import * as RadixAccordion from "@radix-ui/react-accordion";
import styles from "./Accordion.module.css";
import { Icon, Box } from "src/components/design-system";

export type AccordionRootProps = {
  children: React.ReactNode;
  value: string[];
  onValueChange: (value: string[]) => void;
};

const AccordionRoot = ({
  children,
  value,
  onValueChange,
}: AccordionRootProps) => (
  <RadixAccordion.Root
    type="multiple"
    value={value}
    onValueChange={onValueChange}
    className={styles.accordionRoot}
  >
    {children}
  </RadixAccordion.Root>
);

AccordionRoot.displayName = "Accordion";

export type AccordionItemProps = {
  children: React.ReactNode;
  value: string;
};

const AccordionItem = ({ children, value }: AccordionItemProps) => (
  <RadixAccordion.Item
    value={value}
    className={styles.accordionItem}
  >
    {children}
  </RadixAccordion.Item>
);

AccordionItem.displayName = "Accordion.Item";

export type AccordionTriggerProps = {
  children: React.ReactNode;
};

const AccordionTrigger = ({ children }: AccordionTriggerProps) => (
  <RadixAccordion.Header className={styles.accordionHeader} asChild>
    <RadixAccordion.Trigger className={styles.accordionTrigger} asChild>
      <Box>
        {children}
        <div className={styles.accordionChevron}>
          <Icon type="chevron_down" color="default" />
        </div>
      </Box>
    </RadixAccordion.Trigger>
  </RadixAccordion.Header>
);

AccordionTrigger.displayName = "Accordion.Trigger";

export type AccordionContentProps = {
  children: React.ReactNode;
};

const AccordionContent = ({ children }: AccordionContentProps) => (
  <RadixAccordion.Content forceMount className={styles.accordionContent}>
    <div className={styles.accordionContentText}>{children}</div>
  </RadixAccordion.Content>
);

AccordionContent.displayName = "Accordion.Content";

export const Accordion = Object.assign(AccordionRoot, {
  Item: AccordionItem,
  Trigger: AccordionTrigger,
  Content: AccordionContent,
});
