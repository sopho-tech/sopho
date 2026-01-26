import * as RadixAccordion from "@radix-ui/react-accordion";
import styles from "./Accordion.module.css";
import { Icon, Box } from "src/components/design-system";

export type AccordionItemConfig = {
  value: string;
  trigger: React.ReactNode;
  content: React.ReactNode;
};

export type AccordionProps = {
  items: AccordionItemConfig[];
  value: string[];
  onValueChange: (value: string[]) => void;
};

export function Accordion({ items, value, onValueChange }: AccordionProps) {
  const accordionItems = items.map((v: AccordionItemConfig) => {
    return (
      <RadixAccordion.Item
        key={v.value}
        value={v.value}
        className={styles.accordionItem}
      >
        <RadixAccordion.Header className={styles.accordionHeader} asChild>
          <RadixAccordion.Trigger className={styles.accordionTrigger} asChild>
            <Box>
              {v.trigger}
              <div className={styles.accordionChevron}>
                <Icon type="chevron_down" color="default" />
              </div>
            </Box>
          </RadixAccordion.Trigger>
        </RadixAccordion.Header>
        <RadixAccordion.Content forceMount className={styles.accordionContent}>
          <div className={styles.accordionContentText}>{v.content}</div>
        </RadixAccordion.Content>
      </RadixAccordion.Item>
    );
  });
  return (
    <RadixAccordion.Root
      type="multiple"
      value={value}
      onValueChange={onValueChange}
      className={styles.accordionRoot}
    >
      {accordionItems}
    </RadixAccordion.Root>
  );
}
