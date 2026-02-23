import { buttonVariants } from "fumadocs-ui/components/ui/button";
import Image from "next/image";
import Link from "next/link";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "404 | Page Not Found",
};

export default function NotFound() {
  return (
    <>
      <style
        dangerouslySetInnerHTML={{
          __html: `
        body > div > aside,
        body > div > nav,
        [data-sidebar],
        [data-nav],
        nav[aria-label],
        aside[aria-label],
        body aside,
        aside[class*="sidebar"],
        nav[class*="sidebar"] {
          display: none !important;
        }
        body > div > main,
        [role="main"],
        main[class*="main"] {
          margin-left: 0 !important;
          padding-left: 0 !important;
          width: 100% !important;
        }
      `,
        }}
      />
      <div
        className="fixed inset-0 flex items-center justify-center w-screen h-screen z-[9999]"
        style={{
          backgroundColor: "var(--color-fd-background)",
          position: "fixed",
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          width: "100vw",
          height: "100vh",
        }}
      >
        <Image
          src="/icon.svg"
          alt="404"
          width={40}
          height={40}
          className="hidden md:block absolute top-4 left-4"
          style={{ position: "absolute", top: "1rem", left: "1rem" }}
        />
        <div className="w-full max-w-2xl px-6 space-y-6 text-center">
          <div className="space-y-2">
            <h1
              className="font-bold tracking-tighter transition-transform leading-none"
              style={{
                color: "var(--color-fd-primary)",
                fontSize: "6rem",
                fontWeight: 700,
                letterSpacing: "-0.05em",
                lineHeight: 1,
              }}
            >
              404
            </h1>
            <h2
              className="font-light tracking-tighter transition-transform"
              style={{
                color: "var(--color-fd-foreground)",
                fontSize: "2.25rem",
                fontWeight: 300,
                letterSpacing: "-0.05em",
              }}
            >
              Page not found
            </h2>
          </div>
          <p
            className="text-lg"
            style={{
              color: "var(--color-fd-muted-foreground)",
              fontSize: "1.125rem",
            }}
          >
            Oops! You&apos;ve wandered off the Zen path. Let&apos;s guide you
            back to tranquility.
          </p>
          <Link
            className={buttonVariants({
              color: "outline",
            })}
            style={{
              marginTop: "3rem",
              display: "inline-block",
            }}
            href="/"
          >
            Return to Docs
          </Link>
        </div>
      </div>
    </>
  );
}
