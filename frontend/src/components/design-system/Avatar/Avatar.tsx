import styles from "src/components/design-system/Avatar/Avatar.module.css";

export type AvatarProps = {
  name: string;
};

function initialsOf(name: string): string {
  const words = name.trim().split(/\s+/).filter(Boolean);
  const firstWord = words[0] ?? "";
  const lastWord = words.length > 1 ? words[words.length - 1] : "";
  return `${firstWord.charAt(0)}${lastWord.charAt(0)}`.toUpperCase();
}

export function Avatar({ name }: AvatarProps) {
  return (
    <span className={styles.avatar} aria-hidden="true">
      {initialsOf(name)}
    </span>
  );
}
