import { ReactNode } from "react";
import SophoFormDetailsStyles from "src/components/SophoFormDetails/SophoFormDetails.module.css";

type DetailItem = {
  label: string;
  value: ReactNode;
  className?: string;
};

type SophoFormDetailsProps = {
  items: DetailItem[];
  className?: string;
};

export function SophoFormDetails({ items, className }: SophoFormDetailsProps) {
  return (
    <dl
      className={`${SophoFormDetailsStyles.detailsContainer} ${className || ""}`}
    >
      {items.map((item, index) => (
        <div key={index} className={SophoFormDetailsStyles.elementContainer}>
          <dt className={SophoFormDetailsStyles.detailLabel}>{item.label}</dt>
          <dd
            className={`${SophoFormDetailsStyles.detailValue} ${item.className || ""}`}
          >
            {item.value}
          </dd>
        </div>
      ))}
    </dl>
  );
}
