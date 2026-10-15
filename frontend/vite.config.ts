import { playwright } from "@vitest/browser-playwright";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react()],
  resolve: {
    tsconfigPaths: true,
  },
  server: {
    port: 3000,
    proxy: {
      "/api": {
        target: process.env.VITE_API_HOSTNAME || "http://localhost:8000",
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: "dist",
  },
  test: {
    projects: [
      {
        extends: true,
        test: {
          name: "unit",
          environment: "node",
          include: ["src/**/*.test.ts"],
          setupFiles: ["src/test/unit_setup.ts"],
        },
      },
      {
        extends: true,
        publicDir: "src/test/msw/public",
        optimizeDeps: {
          entries: ["src/main.tsx", "src/**/*.test.tsx"],
        },
        test: {
          name: "browser",
          include: ["src/**/*.test.tsx"],
          testTimeout: 30_000,
          setupFiles: ["src/test/browser_setup.ts"],
          browser: {
            enabled: true,
            headless: true,
            provider: playwright(),
            instances: [{ browser: "chromium" }],
            viewport: { width: 1440, height: 900 },
          },
        },
      },
    ],
    coverage: {
      provider: "v8",
      include: ["src/**/*.{ts,tsx}"],
      exclude: [
        "src/**/*.test.{ts,tsx}",
        "src/test/**",
        "src/**/*.d.ts",
        "src/main.tsx",
      ],
      reporter: ["text-summary", ["lcov", { projectRoot: ".." }]],
    },
  },
});
