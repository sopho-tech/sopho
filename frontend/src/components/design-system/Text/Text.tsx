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
  children: React.ReactNode;
};

export function Text({ as = "span", children, color = "default" }: TextProps) {
  const Component = as;
  const className = styles[color];
  return <Component className={className}>{children}</Component>;
}
