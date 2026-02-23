import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import Image from "next/image";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <>
          <Image
            width="24"
            height="24"
            src="/logo.svg"
            aria-label="Logo"
            alt="Logo"
          />
          sopho docs
        </>
      ),
    },
  };
}
