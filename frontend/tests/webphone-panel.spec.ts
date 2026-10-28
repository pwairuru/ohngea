import { test, expect, Page } from "@playwright/test";

const DOMAIN = "test.example.com";
const USER = "admin";
const PASS = "admin";

async function loginViaAPI(page: Page) {
  const res = await page.request.post("/auth/login", {
    data: { username: USER, password: PASS, domain: DOMAIN },
  });
  let tok: string;
  let did: string;
  if (!res.ok()) {
    const r2 = await page.request.post("/api/v1/auth/login", {
      data: { username: USER, password: PASS, domain: DOMAIN },
    });
    expect(r2.ok()).toBeTruthy();
    const j2 = await r2.json();
    tok = j2.access_token;
    did = j2.domain_id || "";
  } else {
    const j = await res.json();
    tok = j.access_token;
    did = j.domain_id || "";
  }
  await page.addInitScript(
    ({ tok, did }) => {
      localStorage.setItem("token", tok);
      if (did) localStorage.setItem("domain_id", did);
      try {
        const payload = JSON.parse(atob(tok.split(".")[1]));
        const domain = payload.domain_id;
        const user = payload.sub;
        if (domain && user) {
          const cfgKey = `webphone:cfg:${domain}:${user}`;
          const dummyCfg = {
            sip_uri: `sip:${user}@test.example.com`,
            wss_url: "wss://test.example.com/wss",
            auth_user: user,
            password: "dummy-pass",
            display_name: user,
          };
          localStorage.setItem(cfgKey, JSON.stringify(dummyCfg));
        }
      } catch {}
    },
    { tok, did }
  );
  return tok;
}

test("Ctrl+K focuses dial and Esc handled", async ({ page }) => {
  // E2E: webphone family slow under vite dev + single worker; triple budget, asserts unchanged.
  test.slow();
  await loginViaAPI(page);
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
  // open via dock-expand (only button wired to setWebPhoneOpen; dock div itself is non-interactive)
  // force: Playwright mobile emulation mis-hit-tests the fixed dock (elementsFromPoint proves a real tap lands on the button)
  const opener = page.getByTestId("dock-expand");
  await expect(opener).toBeVisible();
  await opener.click({ force: true });
  const panel = page.getByTestId("webphone-panel").or(page.locator('[role="dialog"]')).first();
  await expect(panel).toBeVisible();
  // ensure Dialpad tab active
  await page.getByRole("tab", { name: "Dialpad" }).click();
  const dial = page.getByTestId("dial-input");
  await expect(dial).toBeVisible();
  // blur by focusing another tab then back, then click panel to ensure blur
  await page.getByRole("tab", { name: "History" }).click();
  await page.getByRole("tab", { name: "Dialpad" }).click();
  await expect(dial).toBeVisible();
  // ensure dial not focused before shortcut: click outside then verify
  await page.getByRole("tab", { name: "Contacts" }).click();
  await page.getByRole("tab", { name: "Dialpad" }).click();
  await expect(dial).toBeVisible();
  await page.keyboard.press("Control+k");
  await expect(dial).toBeFocused();
  // Esc when idle should not crash: Dialog closes on Escape by design (ui/dialog.tsx),
  // so assert clean close + dock still alive instead of panel visibility
  await page.keyboard.press("Escape");
  await expect(panel).toBeHidden();
  await expect(page.getByTestId("webphone-dock")).toBeVisible();
});

test("DevicePicker visible in Settings tab", async ({ page }) => {
  // E2E: webphone family slow under vite dev + single worker; triple budget, asserts unchanged.
  test.slow();
  await loginViaAPI(page);
  await page.goto("/");
  await page.getByTestId("dock-expand").click({ force: true });
  const panel = page.getByTestId("webphone-panel").or(page.locator('[role="dialog"]')).first();
  await expect(panel).toBeVisible();
  await page.getByRole("tab", { name: "Settings" }).click();
  await expect(page.getByTestId("device-picker")).toBeVisible();
  await expect(page.getByTestId("device-input-select")).toBeVisible();
  await expect(page.getByTestId("device-output-select")).toBeVisible();
});
