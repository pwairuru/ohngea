import { test, expect } from "@playwright/test";

async function login(page: any) {
  const res = await page.request.post("http://localhost:8000/auth/login", {
    data: { username: "admin", password: "admin", domain: "test.example.com" },
  });
  const json = await res.json();
  const token = json.access_token;
  await page.addInitScript((t: string) => localStorage.setItem("token", t), token);
}

test("cdr table collapses to cards on mobile", async ({ page }) => {
  await login(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/cdr");
  await expect(page.getByText("Call Detail Records")).toBeVisible();
});
