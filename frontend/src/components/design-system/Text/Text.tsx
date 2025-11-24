import styles from "src/components/design-system/Text/Text.module.css";

type TextProps = {
  as?: "span" | "div" | "label" | "p";
  color?:
    | "default"
    | "white"
    | "subtle"
    | "disabled"
    | "error"
    | "success"
    | "warning";
  fontSize?:
    | "xs"
    | "sm"
    | "base"
    | "lg"
    | "xl"
    | "2xl"
    | "3xl"
    | "4xl"
    | "5xl"
    | "6xl";
  children: React.ReactNode;
};

export function Text({
  as = "span",
  children,
  color = "default",
  fontSize,
}: TextProps) {
  const Component = as;
  const classNames = [styles[color], fontSize && styles[`fontSize-${fontSize}`]]
    .filter(Boolean)
    .join(" ");
  return <Component className={classNames}>{children}</Component>;
}
