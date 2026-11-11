import * as RadixAvatar from "@radix-ui/react-avatar";
import styles from "src/components/design-system/Avatar/Avatar.module.css";

export type AvatarProps = {
  name: string;
  size?: "sm" | "md";
};

function initialsOf(name: string): string {
  const words = name.trim().split(/\s+/).filter(Boolean);
  const firstWord = words[0] ?? "";
  const lastWord = words.length > 1 ? words[words.length - 1] : "";
  return `${firstWord.charAt(0)}${lastWord.charAt(0)}`.toUpperCase();
}

export function Avatar({ name, size = "md" }: AvatarProps) {
  return (
    <RadixAvatar.Root
      className={`${styles.avatar} ${styles[size]}`}
      aria-hidden="true"
    >
      <RadixAvatar.Fallback className={styles.fallback}>
        {initialsOf(name)}
      </RadixAvatar.Fallback>
    </RadixAvatar.Root>
  );
}
