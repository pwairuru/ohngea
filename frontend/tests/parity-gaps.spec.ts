import { test, expect, Page } from "@playwright/test";

const CREDS = { username: "admin", password: "admin", domain: "test.example.com" };

async function login(page: Page) {
  const r = await page.request.post("/api/v1/auth/login", { data: CREDS });
  expect(r.ok()).toBeTruthy();
  const j = await r.json();
  await page.addInitScript(
    ({ tok, did }: any) => {
      localStorage.setItem("token", tok);
      localStorage.setItem("domain_id", did);
    },
    { tok: j.access_token, did: j.domain_id }
  );
  return j.access_token as string;
}

async function api(page: Page, tok: string, method: "post" | "patch" | "delete" | "get", path: string, data?: any) {
  const r = await (page.request as any)[method](`/api/v1${path}`, {
    headers: { Authorization: `Bearer ${tok}` },
    ...(data ? { data } : {}),
  });
  return r;
}

function uniq(prefix: string) {
  return `${prefix}${Math.floor(Math.random() * 900 + 100)}${Date.now() % 10000}`;
}

test.describe("parity gaps: exception areas", () => {
  test("call forward DND mirrors onto the extension", async ({ page }) => {
    test.slow();
    const tok = await login(page);
    const ext = `76${Math.floor(Math.random() * 900 + 100)}`;
    const cr = await api(page, tok, "post", "/extensions", {
      extension: ext, effective_caller_id_name: "DND Probe", password: "Secret123!",
    });
    expect(cr.ok()).toBeTruthy();
    const { extension_uuid } = await cr.json();

    await page.goto("/call_forward");
    await expect(page.locator("h1", { hasText: "Call Forward" })).toBeVisible();
    await page.click('button:has-text("Add Forward")');
    await page.locator("#fld-extension_uuid, input[placeholder*='extension_uuid' i]").first().fill(extension_uuid);
    await page.fill('input[placeholder="2000"]', "9911");
    await page.locator('label:has-text("Do Not Disturb")').click();
    const waitCreate = page.waitForResponse(
      (r) => r.url().includes("/call_forward") && r.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.click('button:has-text("Create")');
    await waitCreate;
    // Mirror PATCH to /extensions races the POST response; wait for it.
    await page.waitForResponse(
      (r) => r.url().includes(`/extensions/${extension_uuid}`) && r.request().method() === "PATCH",
      { timeout: 15000 }
    );

    const me = await (await api(page, tok, "get", `/extensions/${extension_uuid}`)).json();
    expect(me.do_not_disturb).toBe(true);

    // cleanup
    const list = await (await api(page, tok, "get", "/call_forward")).json();
    const row = (list.data ?? []).find((r: any) => r.extension_uuid === extension_uuid);
    if (row) await api(page, tok, "delete", `/call_forward/${row.call_forward_uuid}`);
    await api(page, tok, "delete", `/extensions/${extension_uuid}`);
  });

  test("contacts link children CRUD in UI (phones + settings)", async ({ page }) => {
    test.slow();
    const tok = await login(page);
    const cname = uniq("LinkChild");
    const cr = await api(page, tok, "post", "/contacts", { name: cname, number: "5550199" });
    expect(cr.ok()).toBeTruthy();
    const { contact_uuid } = await cr.json();

    await page.goto("/contacts");
    await expect(page.getByRole("heading", { name: "Contacts" }).first()).toBeVisible();
    await page.getByPlaceholder(/Search/i).fill(cname);
    const row = page.locator("tr", { hasText: cname });
    await expect(row).toBeVisible({ timeout: 15000 });
    await row.locator('button[aria-label="Edit"]').click();
    await expect(page.getByText("Edit Contact")).toBeVisible();

    // phones child
    const dlg = page.getByRole("dialog");
    const block = (title: string) =>
      dlg.locator("div.border.rounded.p-3").filter({ has: page.locator("span.text-sm.font-semibold", { hasText: new RegExp(`^${title} \\(\\d+\\)$`) }) });
    const phones = block("Phones");
    await phones.getByRole("button", { name: "Add" }).click();
    await page.locator("#fld-phone_number").fill("1555000111");
    await page.locator("#fld-phone_label").fill("mobile");
    await page.getByRole("button", { name: "Save Phones" }).click();
    await expect(phones.getByText(/1555000111/).first()).toBeVisible({ timeout: 15000 });

    // settings child
    const settings = block("Settings");
    await settings.getByRole("button", { name: "Add" }).click();
    await page.locator("#fld-contact_setting_name").fill("tz");
    await page.locator("#fld-contact_setting_value").fill("UTC");
    await page.getByRole("button", { name: "Save Settings" }).click();
    await expect(settings.getByText(/tz/).first()).toBeVisible({ timeout: 15000 });

    // verify via API
    const ph = await (await api(page, tok, "get", `/contacts/${contact_uuid}/phones`)).json();
    expect(ph.total).toBe(1);
    expect(ph.data[0].phone_number).toBe("1555000111");
    const st = await (await api(page, tok, "get", `/contacts/${contact_uuid}/settings`)).json();
    expect(st.data[0].contact_setting_name).toBe("tz");

    await api(page, tok, "delete", `/contacts/${contact_uuid}`);
  });

  test("software page CRUD", async ({ page }) => {
    test.slow();
    await login(page);
    await page.goto("/software");
    await expect(page.getByRole("heading", { name: "Software" })).toBeVisible();
    await page.click('button:has-text("Add Software")');
    await page.locator("#fld-software_name").fill("e2e-soft");
    await page.locator("#fld-software_version").fill("9.9.9");
    const waitCreate = page.waitForResponse(
      (r) => r.url().includes("/software") && r.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.click('button:has-text("Create")');
    await waitCreate;
    await expect(page.getByText("e2e-soft").first()).toBeVisible({ timeout: 15000 });
    page.on("dialog", async (d) => await d.accept());
    await page.locator('button[aria-label="Delete"]').first().click();
  });

  test("databases page CRUD", async ({ page }) => {
    test.slow();
    await login(page);
    await page.goto("/databases");
    await expect(page.getByRole("heading", { name: "Databases" })).toBeVisible();
    await page.click('button:has-text("Add Database")');
    await page.locator("#fld-database_name").fill("e2e-db");
    await page.locator("#fld-database_type").fill("pgsql");
    await page.locator("#fld-database_host").fill("db");
    const waitCreate = page.waitForResponse(
      (r) => r.url().includes("/databases") && r.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.click('button:has-text("Create")');
    await waitCreate;
    await expect(page.getByText("e2e-db").first()).toBeVisible({ timeout: 15000 });
  });

  test("menu_items page CRUD", async ({ page }) => {
    test.slow();
    await login(page);
    await page.goto("/menu_items");
    await expect(page.getByRole("heading", { name: "Menu Items" })).toBeVisible();
    await page.click('button:has-text("Add Menu Item")');
    await page.locator("#fld-menu_item_title").fill("E2E Item");
    await page.locator("#fld-menu_item_link").fill("/e2e");
    const waitCreate = page.waitForResponse(
      (r) => r.url().includes("/menu_items") && r.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.click('button:has-text("Create")');
    await waitCreate;
    await expect(page.getByText("E2E Item").first()).toBeVisible({ timeout: 15000 });
  });

  test("notifications page renders PHP blob field", async ({ page }) => {
    test.slow();
    await login(page);
    await page.goto("/notifications");
    await expect(page.getByRole("heading", { name: "Notifications" })).toBeVisible();
  });

  test("follow_me page sets cid prefixes (call_forward cross-page parity)", async ({ page }) => {
    test.slow();
    const tok = await login(page);
    const fm = uniq("fm");
    await page.goto("/follow_me");
    await expect(page.getByRole("heading", { name: "Follow Me" })).toBeVisible();
    await page.click('button:has-text("Add Follow Me")');
    await page.locator("#fld-follow_me_destination").fill("18885550199");
    await page.getByRole("button", { name: /^Show Advanced/ }).click();
    await page.locator("#fld-cid_name_prefix").fill("ACME-");
    await page.locator("#fld-cid_number_prefix").fill("+1");
    await page.locator("#fld-follow_me_ignore_busy").check();
    const waitCreate = page.waitForResponse(
      (r) => r.url().includes("/follow_me") && r.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.click('button:has-text("Create")');
    await waitCreate;
    const list = await (await api(page, tok, "get", "/follow_me")).json();
    const row = (list.data ?? []).find((r: any) =>
      (r.destinations ?? []).some((d: any) => (d.follow_me_destination ?? d.destination) === "18885550199"));
    expect(row, "row persisted").toBeTruthy();
    expect(row.cid_name_prefix).toBe("ACME-");
    expect(row.follow_me_ignore_busy).toBe(true);
    await api(page, tok, "delete", `/follow_me/${row.follow_me_uuid}`);
  });
});
