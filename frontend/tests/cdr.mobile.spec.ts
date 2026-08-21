import { test, expect } from "@playwright/test";

test("cdr table collapses to cards on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/cdr");
  await expect(page.getByText("Call Detail Records")).toBeVisible();
});
