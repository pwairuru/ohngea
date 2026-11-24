import { test, expect } from "@playwright/test";

async function login(page: any) {
  const res = await page.request.post("/auth/login", {
    data: { username: "admin", password: "admin", domain: "test.example.com" },
  });
  const json = await res.json();
  const token = json.access_token;
  await page.addInitScript((t: string) => localStorage.setItem("token", t), token);
}

test("mobile responsive tables -> cards", async ({ page }) => {
  await login(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/extensions");
  // desktop table hidden on mobile, cards visible — use heading strict
  await expect(page.getByRole("heading", { name: "Extensions" })).toBeVisible();
  await expect(page.locator('[data-testid="extensions-cards"]')).toBeVisible();
});

test("pixel 7 viewport drawer", async ({ page }) => {
  await login(page);
  await page.setViewportSize({ width: 412, height: 915 });
  await page.goto("/");
  await expect(page.locator('[data-testid="mobile-nav-toggle"]')).toBeVisible();
  await page.click('[data-testid="mobile-nav-toggle"]');
  await expect(page.locator('[data-testid="mobile-drawer"]')).toBeVisible();
});
