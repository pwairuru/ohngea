import { test, expect } from "@playwright/test";
import fs from "fs";

test("logo and favicon assets exist", async () => {
  const candidates = [
    "frontend/public/logo.svg",
    "public/logo.svg",
    "frontend/public/favicon.svg",
    "public/favicon.svg",
  ];
  const logoPath = fs.existsSync("frontend/public/logo.svg") ? "frontend/public/logo.svg" : fs.existsSync("public/logo.svg") ? "public/logo.svg" : "frontend/public/logo.svg";
  const faviconPath = fs.existsSync("frontend/public/favicon.svg") ? "frontend/public/favicon.svg" : "public/favicon.svg";
  const icon192Path = fs.existsSync("frontend/public/icon-192.png") ? "frontend/public/icon-192.png" : "public/icon-192.png";
  const icon512Path = fs.existsSync("frontend/public/icon-512.png") ? "frontend/public/icon-512.png" : "public/icon-512.png";
  expect(fs.existsSync(logoPath)).toBeTruthy();
  expect(fs.existsSync(faviconPath)).toBeTruthy();
  expect(fs.existsSync(icon192Path)).toBeTruthy();
  expect(fs.existsSync(icon512Path)).toBeTruthy();
  const logo = fs.readFileSync(logoPath, "utf-8");
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

test("PWA manifest is Ohngea", async ({ page }) => {
  await page.goto("/");
  const manifest = await page.evaluate(async () => {
    const link = document.querySelector('link[rel="manifest"]') as HTMLLinkElement;
    if (!link) return null;
    const res = await fetch(link.href);
    return res.json();
  });
  expect(manifest.name).toBe("Ohngea PBX");
  expect(manifest.short_name).toBe("Ohngea");
  expect(manifest.theme_color).toBe("#14B8A6");
});

test("login shows Ohngea PBX", async ({ page }) => {
  await page.goto("/login");
  await expect(page.getByText("Sign in to Ohngea PBX")).toBeVisible();
});

test("header shows Ohngea lockup", async ({ page }) => {
  await page.goto("/login");
  await expect(page.getByText("SPEAK · CONNECT · OHNGEA")).toBeVisible();
});
