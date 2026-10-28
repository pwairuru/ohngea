import { test, expect } from "@playwright/test";

async function login(page: any) {
  const res = await page.request.post("/auth/login", {
    data: { username: "admin", password: "admin", domain: "test.example.com" },
  });
  const json = await res.json();
  await page.addInitScript((t: string) => localStorage.setItem("token", t), json.access_token);
}

test("command palette opens and navigates", async ({ page }) => {
  await login(page);
  await page.goto("/");
  // PaletteProvider keydown effect attaches after mount; settle on Dashboard first (race fix).
  await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
  await page.keyboard.press("Control+k");
  await expect(page.locator('[data-testid="command-palette"]')).toBeVisible();
  await page.fill('[data-testid="command-input"]', "gate");
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/\/gateways/);
});
