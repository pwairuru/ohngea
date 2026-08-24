import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  testIgnore: "**/*.test.*",
  timeout: 30 * 1000,
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: "list",
  use: {
    baseURL: "http://localhost:8000",
    trace: "on-first-retry",
    headless: true,
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "iPhone 14",
      // use chromium instead of webkit to avoid missing host deps (libmanette) in CI
      use: { ...devices["iPhone 14"], browserName: "chromium" as const },
    },
    {
      name: "Pixel 7",
      use: { ...devices["Pixel 7"] },
    },
  ],
  // webServer disabled — api already running on 8000 serving frontend/dist
  // webServer: {
  //   command: "npm run dev",
  //   port: 5173,
  //   reuseExistingServer: !process.env.CI,
  // },
});
