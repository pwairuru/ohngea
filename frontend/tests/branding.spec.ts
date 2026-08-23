import { test, expect } from "@playwright/test";
import fs from "fs";
import path from "path";

function resolveAsset(p: string): string {
  const base = path.basename(p);
  const candidates = [
    p,
    path.join("public", base),
    path.resolve("frontend/public", base),
    path.resolve("public", base),
    path.join(process.cwd(), "public", base),
    path.join(process.cwd(), "frontend/public", base),
    path.join(process.cwd(), "..", "frontend/public", base),
  ];
  for (const c of candidates) if (fs.existsSync(c)) return c;
  return p;
}

test("logo and favicon assets exist", async () => {
  expect(fs.existsSync(resolveAsset("frontend/public/logo.svg"))).toBeTruthy();
  expect(fs.existsSync(resolveAsset("frontend/public/favicon.svg"))).toBeTruthy();
  expect(fs.existsSync(resolveAsset("frontend/public/icon-192.png"))).toBeTruthy();
  expect(fs.existsSync(resolveAsset("frontend/public/icon-512.png"))).toBeTruthy();
  const logo = fs.readFileSync(resolveAsset("frontend/public/logo.svg"), "utf-8");
  expect(logo).toContain("Ohngea");
  expect(logo).toContain("#14B8A6");
  expect(logo).toContain("currentColor");
});

test("index.html has Ohngea favicon and title", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle(/Ohngea PBX/);
  const favicon = page.locator('link[rel="icon"]');
  await expect(favicon.first()).toHaveAttribute("href", /favicon\.svg/);
});
