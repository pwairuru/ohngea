import { test, expect, Page } from "@playwright/test";
async function login(page:Page){ const r=await page.request.post("/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); let j=await r.json(); let tok=j.access_token, did=j.domain_id; if(!r.ok()){ const r2=await page.request.post("/api/v1/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); const j2=await r2.json(); tok=j2.access_token; did=j2.domain_id;} await page.addInitScript(({tok,did})=>{localStorage.setItem("token",tok); localStorage.setItem("domain_id",did);}, {tok,did}); return {tok, did}; }
test("contacts search filters", async({page})=>{
  // E2E: webphone family slow under vite dev + single worker; triple budget, asserts unchanged.
  test.slow();
  // Self-sufficient: ensure an extension containing "100" exists (fresh DBs seed none).
  const {tok} = await login(page);
  const ext = `100${Math.floor(Math.random()*90+10)}`;
  let created = "";
  const cr = await page.request.post("/api/v1/extensions", {headers:{Authorization:`Bearer ${tok}`}, data:{extension:ext, effective_caller_id_name:"Webphone Contact", password:"Secret123!"}});
  if (cr.ok()) { created = (await cr.json()).extension_uuid; }
  // force: Playwright mobile emulation mis-hit-tests the fixed dock (parent div "intercepts pointer events"; elementsFromPoint proves a real tap lands on the button)
  await page.goto("/"); await page.getByTestId("dock-expand").click({ force: true }); await page.getByRole("tab",{name:"Contacts"}).click(); await expect(page.getByTestId("contacts-search")).toBeVisible(); await page.getByTestId("contacts-search").fill("100"); await expect(page.getByTestId("contacts-list")).toContainText("100");
  if (created) { await page.request.delete(`/api/v1/extensions/${created}`, {headers:{Authorization:`Bearer ${tok}`}}); } });
