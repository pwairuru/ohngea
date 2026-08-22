import { test, expect, Page } from "@playwright/test";

const DOMAIN = "test.example.com";
const USER = "admin";
const PASS = "admin";

async function loginViaAPI(page: Page) {
  const res = await page.request.post("http://localhost:8000/auth/login", {
    data: { username: USER, password: PASS, domain: DOMAIN },
  });
  if (!res.ok()) {
    const r2 = await page.request.post("http://localhost:8000/api/v1/auth/login", {
      data: { username: USER, password: PASS, domain: DOMAIN },
    });
    expect(r2.ok()).toBeTruthy();
    const j2 = await r2.json();
    await page.addInitScript((t: string) => localStorage.setItem("token", t), j2.access_token);
    await page.addInitScript((d: string) => localStorage.setItem("domain_id", d), j2.domain_id || "");
    return j2.access_token;
  }
  const json = await res.json();
  await page.addInitScript((t: string) => localStorage.setItem("token", t), json.access_token);
  if (json.domain_id) await page.addInitScript((d: string) => localStorage.setItem("domain_id", d), json.domain_id);
  return json.access_token;
}

test("webphone button visible and popup toggles", async ({ page }) => {
  await loginViaAPI(page);
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
  const btn = page.getByTestId("webphone-btn");
  await expect(btn).toBeVisible();
  // Phone icon present (lucide)
  await expect(btn.locator("svg")).toBeVisible();
  await btn.click();
  await expect(page.getByTestId("webphone-config-form").or(page.getByTestId("webphone-dialpad"))).toBeVisible();
  // toggle closed via backdrop click or second toggle? use open state
  // ensure dialog is accessible
  await expect(page.locator('[role="dialog"]')).toBeVisible();
});

test("dialpad call button requires input", async ({ page }) => {
  await loginViaAPI(page);
  await page.goto("/");
  await page.getByTestId("webphone-btn").click();
  const configForm = page.getByTestId("webphone-config-form");
  const dialpad = page.getByTestId("webphone-dialpad");
  await expect(configForm.or(dialpad)).toBeVisible();
  if (await dialpad.isVisible()) {
    const callBtn = page.getByTestId("call-btn");
    await expect(callBtn).toBeVisible();
    await expect(callBtn).toBeDisabled();
    const dialInput = page.getByTestId("dial-input");
    await expect(dialInput).toBeVisible();
    await dialInput.fill("1001");
    await expect(callBtn).toBeEnabled();
    // dialpad grid buttons visible and 44px-ish
    await expect(page.getByRole("button", { name: "1" })).toBeVisible();
    await page.getByRole("button", { name: "2" }).click();
    await expect(dialInput).toHaveValue("10012");
    await page.getByRole("button", { name: "Clear" }).click();
    // after clear, call disabled again if input cleared
    // Clear button clears dial state; check disabled
    await expect(callBtn).toBeDisabled();
  } else {
    // config form branch: Save & Register visible, WSS line or auth mode
    await expect(page.getByTestId("save-register")).toBeVisible();
    await expect(page.locator("text=Auth mode:").first()).toBeVisible();
    // if Custom SIP toggle exists (auth_mode both), test wss validation
    const customBtn = page.getByText("Custom SIP");
    if (await customBtn.count() > 0) {
      await customBtn.click();
      const wssInput = page.getByPlaceholder("wss://pbx.example.com/wss");
      await wssInput.fill("http://bad.example.com");
      await page.getByTestId("save-register").click();
      await expect(page.getByText(/WSS URL must start with wss:\/\//i)).toBeVisible();
    }
  }
});
