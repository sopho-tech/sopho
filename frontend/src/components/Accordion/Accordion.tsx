import * as RadixAccordion from "@radix-ui/react-accordion";
import styles from "./Accordion.module.css";

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
        <RadixAccordion.Header className={styles.accordionHeader}>
          <RadixAccordion.Trigger className={styles.accordionTrigger}>
            {v.trigger}
            <span
              className={`material-symbols-outlined ${styles.accordionChevron}`}
            >
              keyboard_arrow_down
            </span>
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
