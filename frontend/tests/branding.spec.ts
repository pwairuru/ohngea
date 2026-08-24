import { test, expect } from "@playwright/test";
import fs from "fs";

test("logo and favicon assets exist", async () => {
  expect(fs.existsSync("frontend/public/logo.svg")).toBeTruthy();
  expect(fs.existsSync("frontend/public/favicon.svg")).toBeTruthy();
  expect(fs.existsSync("frontend/public/icon-192.png")).toBeTruthy();
  expect(fs.existsSync("frontend/public/icon-512.png")).toBeTruthy();
  const logo = fs.readFileSync("frontend/public/logo.svg", "utf-8");
  expect(logo).toContain("Ohngea");
  expect(logo).toContain("#14B8A6");
  expect(logo).toContain("currentColor");
});

// pending Task 2 — title is still "Ohngea PBX" until Task 2 updates index.html
test.skip("index.html has Ohngea favicon and title", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle(/Ohngea PBX/);
  const favicon = page.locator('link[rel="icon"]');
  await expect(favicon.first()).toHaveAttribute("href", /favicon\.svg/);
});
