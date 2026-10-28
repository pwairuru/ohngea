import { test, expect, Page } from "@playwright/test";
async function login(page:Page){ const r=await page.request.post("/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); let j=await r.json(); let tok=j.access_token, did=j.domain_id; if(!r.ok()){ const r2=await page.request.post("/api/v1/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); const j2=await r2.json(); tok=j2.access_token; did=j2.domain_id;} await page.addInitScript(({tok,did})=>{localStorage.setItem("token",tok); localStorage.setItem("domain_id",did);}, {tok,did}); }
test("contacts search filters", async({page})=>{
  // E2E: webphone family slow under vite dev + single worker; triple budget, asserts unchanged.
  test.slow();
  // force: Playwright mobile emulation mis-hit-tests the fixed dock (parent div "intercepts pointer events"; elementsFromPoint proves a real tap lands on the button)
  await login(page); await page.goto("/"); await page.getByTestId("dock-expand").click({ force: true }); await page.getByRole("tab",{name:"Contacts"}).click(); await expect(page.getByTestId("contacts-search")).toBeVisible(); await page.getByTestId("contacts-search").fill("100"); await expect(page.getByTestId("contacts-list")).toContainText("100"); });
