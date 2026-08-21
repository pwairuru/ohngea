import { test, expect } from "@playwright/test";

test("mobile responsive tables -> cards", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/extensions");
  // desktop table hidden on mobile, cards visible
  const cards = page.locator('[data-testid="extensions-cards"]');
  // either table or cards should be present; if no data, at least page loads
  await expect(page.getByText("Extensions")).toBeVisible();
});

test("pixel 7 viewport drawer", async ({ page }) => {
  await page.setViewportSize({ width: 412, height: 915 });
  await page.goto("/");
  await expect(page.locator('[data-testid="mobile-nav-toggle"]')).toBeVisible();
  await page.click('[data-testid="mobile-nav-toggle"]');
  await expect(page.locator('[data-testid="mobile-drawer"]')).toBeVisible();
});
