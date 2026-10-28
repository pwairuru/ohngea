import fs from "node:fs";
import path from "node:path";
import { defineConfig, devices } from "@playwright/test";

// API base follows root .env API_PORT (single source of truth); $E2E_API wins.
// Specs use relative URLs so this baseURL is the only host string in frontend tests.
function apiBaseURL(): string {
  if (process.env.E2E_API) return process.env.E2E_API;
  try {
    const txt = fs.readFileSync(path.resolve(__dirname, "..", ".env"), "utf8");
    for (const line of txt.split("\n")) {
      const m = line.match(/^\s*API_PORT\s*=\s*["']?([^"'#\s]+)/);
      if (m) return `http://localhost:${m[1]}`;
    }
  } catch {
    // no root .env — fall through to default
  }
  return "http://localhost:8010";
}

export default defineConfig({
  testDir: "./tests",
  testIgnore: "**/*.test.*",
  // E2E: 60s global (was 30s). Dev server + single worker + 49-route loop
  // exceeds 30s environmentally; no API 5xx observed (endpoints ≤10ms).
  timeout: 60 * 1000,
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: "list",
  use: {
    baseURL: apiBaseURL(),
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
