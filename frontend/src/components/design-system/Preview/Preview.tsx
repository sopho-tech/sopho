import styles from "src/components/design-system/Preview/Preview.module.css";

type HtmlPreviewProps = {
  title: string;
  html: string;
};

export function HtmlPreview({ title, html }: HtmlPreviewProps) {
  return (
    <iframe
      title={title}
      srcDoc={html}
      sandbox=""
      className={`${styles.surface} ${styles.frame}`}
    />
  );
}

type TextPreviewProps = {
  "aria-label": string;
  text: string;
};

export function TextPreview({
  "aria-label": ariaLabel,
  text,
}: TextPreviewProps) {
  return (
    <pre aria-label={ariaLabel} className={`${styles.surface} ${styles.text}`}>
      {text}
    </pre>
  );
}
