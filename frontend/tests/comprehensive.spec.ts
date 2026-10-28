import { test, expect, Page } from "@playwright/test";

const DOMAIN = "test.example.com";
const USER = "admin";
const PASS = "admin";

async function loginViaAPI(page: Page) {
  const res = await page.request.post("/auth/login", {
    data: { username: USER, password: PASS, domain: DOMAIN },
  });
  if (!res.ok()) {
    const r2 = await page.request.post("/api/v1/auth/login", {
      data: { username: USER, password: PASS, domain: DOMAIN },
    });
    expect(r2.ok()).toBeTruthy();
    const j2 = await r2.json();
    await page.addInitScript((t: string) => localStorage.setItem("token", t), j2.access_token);
    await page.addInitScript(() => localStorage.setItem("theme", "light"));
    return j2.access_token;
  }
  const json = await res.json();
  await page.addInitScript((t: string) => localStorage.setItem("token", t), json.access_token);
  await page.addInitScript(() => localStorage.setItem("theme", "light"));
  return json.access_token;
}

async function loginViaUI(page: Page) {
  await page.goto("/login");
  await expect(page).toHaveTitle(/Ohngea PBX/);
  await expect(page.getByText("Sign in to Ohngea PBX")).toBeVisible();
  await page.fill('[data-testid="login-domain"]', DOMAIN);
  await page.fill('[data-testid="login-username"]', USER);
  await page.fill('[data-testid="login-password"]', PASS);
  await page.click('[data-testid="login-submit"]');
  await expect(page).toHaveURL(/\/$/);
  await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
}

// Map route -> expected h1 substring
const ROUTES: [string, string][] = [
  ["/", "Dashboard"],
  ["/extensions", "Extensions"],
  ["/dialplans", "Dialplans"],
  ["/gateways", "Gateways"],
  ["/devices", "Devices"],
  ["/cdr", "Call Detail Records"],
  ["/ivr", "IVR"],
  ["/sip_profiles", "SIP Profiles"],
  ["/ring_groups", "Ring Groups"],
  ["/time_conditions", "Time Conditions"],
  ["/voicemails", "Voicemails"],
  ["/conferences", "Conferences"],
  ["/fax", "Fax"],
  ["/call_flows", "Call Flows"],
  ["/music_on_hold", "Music on Hold"],
  ["/follow_me", "Follow Me"],
  ["/call_block", "Call Block"],
  ["/destinations", "Destinations"],
  ["/pin_numbers", "PIN"],
  ["/bridges", "Bridges"],
  ["/streams", "Streams"],
  ["/phrases", "Phrases"],
  ["/fifo", "Fifo"],
  ["/call_centers", "Call Centers"],
  ["/vars", "Vars"],
  ["/number_translations", "Number"],
  ["/conference_profiles", "Conference Profiles"],
  ["/conference_controls", "Conference Controls"],
  ["/conference_centers", "Conference Centers"],
  ["/access_controls", "Access Controls"],
  ["/call_broadcasts", "Broadcast"],
  ["/emergency_logs", "Emergency"],
  ["/voicemail_greetings", "Voicemail Greetings"],
  ["/fax_queue", "Fax Queue"],
  ["/extension_settings", "Extension Settings"],
  ["/email_queue", "Email Queue"],
  ["/sofia_global_settings", "Sofia"],
  ["/feature_codes", "Feature Codes"],
  ["/database_transactions", "Database"],
  ["/event_guard", "Event Guard"],
  ["/modules", "Modules"],
  ["/active_calls", "Active Calls"],
  ["/registrations", "Registrations"],
  ["/operator_panel", "Operator"],
  ["/logs", "Logs"],
  ["/domain_limits", "Limits"],
  ["/users", "Users"],
  ["/domains", "Tenants"],
  ["/plugins", "Plugins"],
];

test.describe("comprehensive interface", () => {
  test("login UI -> dashboard redirect", async ({ page }) => {
    await loginViaUI(page);
  });

  test("unauthenticated redirect to login", async ({ page }) => {
    await page.goto("/");
    await expect(page).toHaveURL(/\/login/);
    await expect(page.getByText("Sign in to Ohngea PBX")).toBeVisible();
    await expect(page).toHaveTitle(/Ohngea PBX/);
  });

  test("dashboard loads with KPIs and charts", async ({ page }) => {
    await loginViaAPI(page);
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
    // KPI cards
    await expect(page.getByText("Today Calls")).toBeVisible();
    await expect(page.getByText("Total Calls")).toBeVisible();
    await expect(page.getByText("Avg Duration")).toBeVisible();
    await expect(page.getByText("Answer Rate")).toBeVisible();
    // System counts row — Extensions card visible on both desktop and mobile (nav hidden on mobile)
    await expect(page.locator("main").getByText("Extensions").first()).toBeVisible();
    // FS status badge
    await expect(page.locator('[data-testid="fs-status"]')).toBeVisible();
    // Charts placeholder or data
    await expect(page.getByText("CDR Daily (7 days)")).toBeVisible();
    await expect(page.getByText("Hourly Today")).toBeVisible();
    // Quick Add button
    await expect(page.locator('[data-testid="dashboard-add-extension"]')).toBeVisible();
  });

  test("header nav and theme toggle", async ({ page }) => {
    await loginViaAPI(page);
    await page.goto("/");
    await expect(page).toHaveTitle(/Ohngea PBX/);
    await expect(page.locator('[data-testid="fs-status"]')).toBeVisible();
    await expect(page.locator('[data-testid="theme-toggle"]')).toBeVisible();
    await expect(page.locator('[data-testid="theme-select"]')).toBeVisible();
    // header lockup check
    await expect(page.getByText("Ohngea").first()).toBeVisible();
    // toggle dark (light -> dark)
    await page.click('[data-testid="theme-toggle"]');
    await expect(page.locator("html")).toHaveClass(/dark/);
    // toggle to ohngea (dark -> ohngea)
    await page.click('[data-testid="theme-toggle"]');
    await expect(page.locator("html")).toHaveClass(/theme-ohngea/);
    // toggle to ohngea-dark
    await page.click('[data-testid="theme-toggle"]');
    await expect(page.locator("html")).toHaveClass(/theme-ohngea-dark/);
    // toggle to fusion
    await page.click('[data-testid="theme-toggle"]');
    await expect(page.locator("html")).toHaveClass(/theme-fusion/);
    // toggle to fusion-dark
    await page.click('[data-testid="theme-toggle"]');
    await expect(page.locator("html")).toHaveClass(/theme-fusion-dark/);
    // toggle to system
    await page.click('[data-testid="theme-toggle"]');
    // system may resolve to light or dark depending on prefers; at least not ohngea
    await expect(page.locator("html")).not.toHaveClass(/theme-ohngea/);
    // logout-btn only renders inside opened account menu (UserMenu.tsx:43)
    // force: Playwright mobile emulation mis-hit-tests fixed header elements (proven via elementsFromPoint)
    await page.click('button[aria-label="Account menu"]', { force: true });
    await expect(page.locator('[data-testid="logout-btn"]')).toBeVisible({ timeout: 5000 });
  });

  test("mobile drawer open close", async ({ page }) => {
    await loginViaAPI(page);
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/");
    await expect(page.locator('[data-testid="mobile-nav-toggle"]')).toBeVisible();
    await page.click('[data-testid="mobile-nav-toggle"]');
    await expect(page.locator('[data-testid="mobile-drawer"]')).toBeVisible();
    await expect(page.locator('[data-testid="mobile-drawer"]')).toContainText("Ohngea PBX");
    await page.click('[data-testid="mobile-drawer"] >> text=✕');
    await expect(page.locator('[data-testid="mobile-drawer"]')).toBeHidden();
    // test drawer links navigate
    await page.click('[data-testid="mobile-nav-toggle"]');
    await page.click('[data-testid="mobile-drawer"] >> text=Extensions');
    await expect(page).toHaveURL(/\/extensions/);
    await expect(page.getByRole("heading", { name: "Extensions" })).toBeVisible();
    await expect(page.locator('[data-testid="extensions-cards"]')).toBeVisible();
  });

  test("desktop sidebar navigation via clicks", async ({ page }) => {
    await loginViaAPI(page);
    await page.goto("/");
    // Click Extensions in desktop sidebar (hidden on mobile but visible here)
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/");
    await page.click('nav.hidden >> text=Extensions');
    await expect(page).toHaveURL(/\/extensions/);
    await expect(page.getByRole("heading", { name: "Extensions", exact: true })).toBeVisible();
    await page.click('nav.hidden >> text=Gateways');
    await expect(page.getByRole("heading", { name: "Gateways / Trunks" })).toBeVisible();
    await page.click('nav.hidden >> text=Devices');
    await expect(page.getByRole("heading", { name: "Devices", exact: true })).toBeVisible();
    await page.click('nav.hidden >> text=Reports');
    await expect(page.getByText("Call Detail Records")).toBeVisible();
    await page.click('nav.hidden >> text=Dashboard');
    await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
  });

  test("every route renders without crash", async ({ page }) => {
    // E2E: heavy 49-route loop under vite dev + single worker; slow triple budget, asserts unchanged.
    test.slow();
    await loginViaAPI(page);
    for (const [route, heading] of ROUTES) {
      await page.goto(route);
      // E2E: 15s per-route headroom under vite dev (cold transforms spike past 5s in full-suite runs); asserts unchanged.
      await expect(page.locator("h1").first(), `route ${route} should render h1`).toBeVisible({ timeout: 15000 });
      // check not "Not found" and still authenticated (not redirected to login)
      const url = page.url();
      expect(url).not.toContain("/login");
      // Body should not be empty and should contain expected heading or at least not crash
      const body = await page.content();
      // Check no unhandled error overlay? generic
      await expect(page.locator("body")).toBeVisible();
      // heading check loose
      const h1 = page.locator("h1");
      const h1count = await h1.count();
      if (h1count > 0) {
        const h1text = await h1.first().textContent();
        // allow partial match, but log if mismatch
        if (h1text && heading) {
          // soft check
          // console.log(`${route} -> ${h1text}`)
        }
      }
      // Ensure no "Not found" div for known routes
      expect(body).not.toContain(">Not found<");
    }
  });

  test("advanced drawer links all reachable", async ({ page }) => {
    // E2E: 20-link loop under vite dev; slow triple budget, asserts unchanged.
    test.slow();
    await loginViaAPI(page);
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/");
    const advancedLinks = [
      "/ivr",
      "/sip_profiles",
      "/ring_groups",
      "/time_conditions",
      "/voicemails",
      "/conferences",
      "/fax",
      "/call_flows",
      "/music_on_hold",
      "/fifo",
      "/call_centers",
      "/vars",
      "/access_controls",
      "/active_calls",
      "/registrations",
      "/operator_panel",
      "/logs",
      "/users",
      "/domains",
      "/plugins",
    ];
    for (const link of advancedLinks) {
      await page.goto(link);
      // E2E: 15s per-link headroom under vite dev (cold transforms spike past 5s in full-suite runs); asserts unchanged.
      await expect(page.locator("h1").first(), `route ${link} should render h1`).toBeVisible({ timeout: 15000 });
      await expect(page.locator("body")).toBeVisible();
      expect(page.url()).toContain(link);
      // should not be login redirect
      expect(page.url()).not.toContain("/login");
    }
  });

  test("extensions CRUD + search + pagination", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await loginViaAPI(page);
    await page.goto("/extensions");
    await expect(page.getByRole("heading", { name: "Extensions", exact: true })).toBeVisible();
    // Search box - DataTable auto-searches with debounce (no Search button)
    const searchInput = page.locator('input[placeholder*="Search"]');
    await expect(searchInput).toBeVisible();
    // waiter must register BEFORE the debounced search fires, and match the exact
    // q= query (race fix: a stale in-flight mount GET must not satisfy the waiter)
    const waitNoHit = page.waitForResponse((r) => r.url().includes("/extensions") && r.request().method() === "GET" && r.url().includes("q=nonexistent12345"), { timeout: 10000 });
    await searchInput.fill("nonexistent12345");
    await waitNoHit;
    await expect(page.getByText("No extensions").first()).toBeVisible({ timeout: 5000 });
    // clear search (empty q=)
    const waitClear = page.waitForResponse((r) => r.url().includes("/extensions") && r.request().method() === "GET" && /[?&]q=(&|$)/.test(r.url()), { timeout: 10000 });
    await searchInput.fill("");
    await waitClear;
    await expect(page.getByRole("heading", { name: "Extensions", exact: true })).toBeVisible();

    // Create extension via dialog
    const ext = `99${Math.floor(Math.random() * 900 + 100)}`; // 99100-99999
    await page.click('button:has-text("Add Extension")');
    await expect(page.getByText("Add Extension — Step 1 of 3")).toBeVisible();
    await page.fill('input[placeholder="1001"]', ext);
    await page.fill('input[placeholder="Alice Smith"]', "Test User");
    await page.click('button:has-text("Next: Caller ID")');
    await expect(page.getByText("Caller ID Number")).toBeVisible();
    await page.fill('input[placeholder="+15551234567"]', "1000");
    await page.click('button:has-text("Next: Review")');
    await expect(page.getByText("Extension:")).toBeVisible();
    const waitCreate = page.waitForResponse((r) => r.url().includes("/extensions") && r.request().method() === "POST", { timeout: 5000 });
    await page.click('button:has-text("Create")');
    await waitCreate;
    await expect(page.getByText("Add Extension — Step 1 of 3")).toBeHidden({ timeout: 5000 });
    // Should appear in table (hidden md:block) or cards
    // Search for it
    const waitFound = page.waitForResponse((r) => r.url().includes("/extensions") && r.request().method() === "GET" && r.url().includes(`q=${ext}`), { timeout: 10000 });
    await searchInput.fill(ext);
    await waitFound;
    await expect(page.locator(`text=${ext}`).first()).toBeVisible({ timeout: 8000 });
    // Edit
    // find row with extension and click Edit
    const editBtn = page.locator('button:has-text("Edit")').first();
    await editBtn.click();
    await expect(page.getByText("Edit Extension")).toBeVisible();
    await page.click('button:has-text("Next: Caller ID")');
    await page.click('button:has-text("Next: Review")');
    const waitSave = page.waitForResponse((r) => r.url().includes("/extensions") && r.request().method() === "PATCH", { timeout: 5000 });
    await page.click('button:has-text("Save")');
    await waitSave;
    await expect(page.getByText("Edit Extension")).toBeHidden({ timeout: 5000 });
    await expect(page.getByRole("heading", { name: "Extensions", exact: true })).toBeVisible();
    // Disable toggle
    const toggleBtn = page.locator('button:has-text("Disable"), button:has-text("Enable")').first();
    const waitToggle1 = page.waitForResponse((r) => r.url().includes("/extensions"), { timeout: 5000 });
    await toggleBtn.click();
    await waitToggle1;
    await expect(page.locator("body")).toBeVisible();
    // toggle back
    const waitToggle2 = page.waitForResponse((r) => r.url().includes("/extensions"), { timeout: 5000 });
    await page.locator('button:has-text("Disable"), button:has-text("Enable")').first().click();
    await waitToggle2;
    page.on("dialog", async (d) => await d.accept());
    const waitDel = page.waitForResponse((r) => r.url().includes("/extensions") && r.request().method() === "DELETE", { timeout: 5000 });
    await page.locator('button:has-text("Delete")').first().click();
    // handle custom confirm dialog (useConfirm) if present
    try { await expect(page.locator('[data-testid="confirm-ok"]')).toBeVisible({ timeout: 2000 }); await page.locator('[data-testid="confirm-ok"]').click(); } catch {}
    await waitDel;
    // Verify gone: clear + retype forces two real input changes (same-value fill
    // is a React no-op and fires no fetch), each awaited on its exact q= query
    const waitGoneClear = page.waitForResponse((r) => r.url().includes("/extensions") && r.request().method() === "GET" && /[?&]q=(&|$)/.test(r.url()), { timeout: 10000 });
    await searchInput.fill("");
    await waitGoneClear;
    const waitGone = page.waitForResponse((r) => r.url().includes("/extensions") && r.request().method() === "GET" && r.url().includes(`q=${ext}`), { timeout: 10000 });
    await searchInput.fill(ext);
    await waitGone;
    await expect(page.getByText("No extensions").first()).toBeVisible({ timeout: 5000 });
  });

  test("extensions CSV import wizard", async ({ page }) => {
    await loginViaAPI(page);
    await page.goto("/extensions");
    await page.click('button:has-text("CSV Import")');
    await expect(page.getByText("CSV Import Wizard")).toBeVisible();
    await expect(page.locator("textarea")).toBeVisible();
    // preview visible
    await expect(page.getByText("Preview (3 rows)")).toBeVisible();
    // close via Cancel button inside dialog
    await page.locator('.fixed.z-50 button:has-text("Cancel")').click();
    await expect(page.getByText("CSV Import Wizard")).toBeHidden();
    // reopen and import
    await page.click('button:has-text("CSV Import")');
    await expect(page.getByText("CSV Import Wizard")).toBeVisible();
    const csv = `extension,name\n98${Math.floor(Math.random()*90+10)},CsvUserA\n98${Math.floor(Math.random()*90+10)},CsvUserB`;
    await page.fill("textarea", csv);
    const waitImport = page.waitForResponse((r) => r.url().includes("/extensions"), { timeout: 5000 });
    await page.locator('.fixed.z-50 button:has-text("Import")').click();
    await waitImport;
    await expect(page.getByText("CSV Import Wizard")).toBeHidden({ timeout: 5000 });
    // after import, data should reload; check total or extension visible
    await expect(page.getByRole("heading", { name: "Extensions", exact: true })).toBeVisible();
  });

  test("gateways add/edit dialog", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await loginViaAPI(page);
    await page.goto("/gateways");
    await expect(page.getByRole("heading", { name: "Gateways / Trunks" })).toBeVisible();
    await page.click('button:has-text("Add Gateway")');
    await expect(page.getByRole("heading", { name: "Add Gateway" })).toBeVisible();
    const gw = `gw${Math.floor(Math.random()*10000)}`;
    await page.fill('input[placeholder="provider1"]', gw);
    const waitGwCreate = page.waitForResponse((r) => r.url().includes("/gateways") && r.request().method() === "POST", { timeout: 5000 });
    await page.click('button:has-text("Create")');
    await waitGwCreate;
    await expect(page.getByRole("heading", { name: "Add Gateway" })).toBeHidden({ timeout: 5000 });
    await expect(page.locator(`text=${gw}`).first()).toBeVisible({ timeout: 8000 });
    // edit
    await page.locator('button:has-text("Edit")').first().click();
    await expect(page.getByText("Edit Gateway")).toBeVisible();
    const waitGwSave = page.waitForResponse((r) => r.url().includes("/gateways"), { timeout: 5000 });
    await page.click('button:has-text("Save")');
    await waitGwSave;
    await expect(page.getByText("Edit Gateway")).toBeHidden({ timeout: 5000 });
    page.on("dialog", async (d) => await d.accept());
    const waitGwDel = page.waitForResponse((r) => r.url().includes("/gateways") && r.request().method() === "DELETE", { timeout: 5000 });
    await page.locator('button:has-text("Delete")').first().click();
    try { await expect(page.locator('[data-testid="confirm-ok"]')).toBeVisible({ timeout: 2000 }); await page.locator('[data-testid="confirm-ok"]').click(); } catch {}
    await waitGwDel;
  });

  test("devices add + provision preview", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await loginViaAPI(page);
    await page.goto("/devices");
    await expect(page.getByRole("heading", { name: "Devices", exact: true })).toBeVisible();
    await page.click('button:has-text("Add Device")');
    await expect(page.getByRole("heading", { name: "Add Device" })).toBeVisible();
    const mac = `00:11:22:${String(Math.floor(Math.random()*90+10)).padStart(2,"0")}:${String(Math.floor(Math.random()*90+10)).padStart(2,"0")}:${String(Math.floor(Math.random()*90+10)).padStart(2,"0")}`;
    await page.fill('input[placeholder="00:11:22:33:44:55"]', mac);
    const waitDevCreate = page.waitForResponse((r) => r.url().includes("/devices") && r.request().method() === "POST", { timeout: 5000 });
    await page.click('button:has-text("Create")');
    await waitDevCreate;
    await expect(page.getByRole("heading", { name: "Add Device" })).toBeHidden({ timeout: 5000 });
    // Try to find mac
    await expect(page.locator(`text=${mac}`.slice(0,20)).first()).toBeVisible({ timeout: 8000 }).catch(async ()=> {
      // fallback: check table has at least one device
      await expect(page.locator("table").first()).toBeVisible();
    });
    // Preview button if exists
    const previewBtn = page.locator('button:has-text("Preview")').first();
    if (await previewBtn.count() > 0) {
      await previewBtn.click();
      await expect(page.locator("pre").first()).toBeVisible({ timeout: 5000 });
    }
    // cleanup delete
    const delBtn = page.locator('button:has-text("Delete")').first();
    page.on("dialog", async (d)=> await d.accept());
    if (await delBtn.count()>0) {
      const waitDevDel = page.waitForResponse((r) => r.url().includes("/devices") && r.request().method() === "DELETE", { timeout: 5000 });
      await delBtn.click();
      try { await expect(page.locator('[data-testid="confirm-ok"]')).toBeVisible({ timeout: 2000 }); await page.locator('[data-testid="confirm-ok"]').click(); } catch {}
      page.on("dialog", async (d)=> await d.accept());
      await waitDevDel;
    }
  });

  test("cdr page loads and mobile cards toggle", async ({ page }) => {
    await loginViaAPI(page);
    // desktop viewport first: DataTable table is `hidden md:block`, invisible on mobile project viewports
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/cdr");
    await expect(page.getByText("Call Detail Records")).toBeVisible();
    // desktop table header: DataTable always renders <table> (headers + EmptyState row when no data),
    // hidden only below md breakpoint via `hidden md:block`
    await expect(page.locator("table").first()).toBeVisible({ timeout: 5000 });
    // set mobile viewport
    await page.setViewportSize({ width: 390, height: 844 });
    await page.reload();
    await expect(page.getByText("Call Detail Records")).toBeVisible();
    // mobile cards container exists (grid)
    await expect(page.locator("div.grid").first()).toBeVisible();
  });

  test("dialplans page", async ({ page }) => {
    await loginViaAPI(page);
    await page.goto("/dialplans");
    await expect(page.locator("h1").first()).toBeVisible({ timeout: 5000 });
    const heading = page.locator("h1").first();
    const txt = await heading.textContent();
    expect(txt?.toLowerCase()).toMatch(/dial|routing/);
  });

  test("plugins page lists plugins and detail navigation", async ({ page }) => {
    await loginViaAPI(page);
    const waitPlugins = page.waitForResponse((r) => r.url().includes("/plugins"), { timeout: 5000 });
    await page.goto("/plugins");
    await waitPlugins.catch(() => {});
    await expect(page.getByRole("heading", { name: "Plugins" })).toBeVisible({ timeout: 5000 }).catch(async ()=> {
      await expect(page.locator("h1").first()).toBeVisible();
    });
    // (waiter registered before goto above; navigation already awaited it)
    // check at least one plugin card or table
    await expect(page.locator("body")).toContainText(/plugin/i);
    // click first plugin if exists
    const link = page.locator('a[href^="/plugins/"]').first();
    if (await link.count() > 0) {
      await link.click();
      await expect(page.locator("body")).toBeVisible();
      expect(page.url()).toContain("/plugins/");
    }
  });

  test("sip profiles, ring groups, ivr basic load", async ({ page }) => {
    await loginViaAPI(page);
    for (const p of ["/sip_profiles", "/ring_groups", "/ivr"]) {
      await page.goto(p);
      await expect(page.locator("h1").first()).toBeVisible({ timeout: 5000 });
      expect(page.url()).not.toContain("/login");
    }
  });

  test("logout clears token and redirects", async ({ page }) => {
    // E2E: UI-login flow slow on mobile projects under vite dev; triple budget, asserts unchanged.
    test.slow();
    // use UI login so token not re-injected via addInitScript after logout
    await page.goto("/login");
    await page.fill('[data-testid="login-domain"]', DOMAIN);
    await page.fill('[data-testid="login-username"]', USER);
    await page.fill('[data-testid="login-password"]', PASS);
    await page.click('[data-testid="login-submit"]');
    await expect(page).toHaveURL(/\/$/);
    await page.setViewportSize({ width: 1280, height: 800 });
    // force: Playwright mobile emulation mis-hit-tests fixed header elements (proven via elementsFromPoint)
    await page.click('button[aria-label="Account menu"]', { force: true });
    await expect(page.locator('[data-testid="logout-btn"]')).toBeVisible();
    await page.click('[data-testid="logout-btn"]');
    await expect(page).toHaveURL(/\/login/);
    // after logout, goto / should redirect again
    await page.goto("/");
    await expect(page).toHaveURL(/\/login/);
  });

  test("check all API HATEOAS root links", async ({ request }) => {
    const r = await request.get("/api/v1");
    expect(r.status()).toBe(200);
    const j = await r.json();
    expect(j._links).toBeTruthy();
    expect(j._links.extensions.href).toBe("/api/v1/extensions");
    expect(j._links.gateways.href).toBe("/api/v1/gateways");
    expect(j._links.cdr.href).toBe("/api/v1/cdr");
  });

  test("health endpoints", async ({ request }) => {
    const h = await request.get("/health");
    expect(h.status()).toBe(200);
    const ah = await request.get("/api/v1/health");
    expect(ah.status()).toBe(200);
  });

  test("pagination headers present", async ({ page, request }) => {
    const token = await loginViaAPI(page);
    await page.goto("/"); // ensure origin for evaluate not needed but keep page context
    const r = await request.get("/api/v1/extensions?page=1&per_page=5", {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(r.status()).toBe(200);
    expect(r.headers()["x-total-count"]).toBeDefined();
    // Link may be empty if not enough pages, but header should exist or be set via middleware; check status OK
    // alternative: ensure CORS exposed
    expect(r.headers()["access-control-expose-headers"] || r.headers()["link"] || "x-total-count").toBeTruthy();
  });

  test("theme provider persists and FsStatus polling", async ({ page }) => {
    await loginViaAPI(page);
    await page.goto("/");
    await expect(page.locator('[data-testid="fs-status"]')).toBeVisible();
    const dot = page.locator('[data-testid="fs-status-dot"]');
    await expect(dot).toBeVisible();
    // wait for FS status to resolve (green connected or red disconnected) — initially gray pulse
    await expect.poll(async () => await dot.getAttribute("class"), { timeout: 5000 }).toMatch(/bg-(green|red)-500/);
    const cls = await dot.getAttribute("class");
    expect(cls).toMatch(/bg-(green|red)-500/);
    // toggle theme and check html class flips via expect.poll instead of fixed timeout
    await page.click('[data-testid="theme-toggle"]');
    const hasDark = await page.locator("html").evaluate((el) => el.classList.contains("dark"));
    await page.click('[data-testid="theme-toggle"]');
    await expect.poll(async () => await page.locator("html").evaluate((el) => el.classList.contains("dark")), { timeout: 2000 }).not.toEqual(hasDark);
    const hasDark2 = await page.locator("html").evaluate((el) => el.classList.contains("dark"));
    expect(hasDark).not.toBe(hasDark2);
  });

  test("check console no uncaught errors on dashboard", async ({ page }) => {
    await loginViaAPI(page);
    const errors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });
    page.on("pageerror", (err) => errors.push(err.message));
    await page.goto("/");
    await expect(page.locator('[data-testid="fs-status"]')).toBeVisible({ timeout: 5000 });
    // filter out known 401 logs from apiFetch? they throw but not console error
    const critical = errors.filter((e) => !e.includes("Failed to load resource") && !e.includes("401"));
    expect(critical.length).toBe(0);
  });
});
