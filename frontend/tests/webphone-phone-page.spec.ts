import { test, expect, Page } from "@playwright/test";
async function login(p:Page){ const r=await p.request.post("/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); let j=await r.json(); let tok=j.access_token, did=j.domain_id; if(!r.ok()){ const r2=await p.request.post("/api/v1/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); const j2=await r2.json(); tok=j2.access_token; did=j2.domain_id;} await p.addInitScript(({tok,did})=>{localStorage.setItem("token",tok); localStorage.setItem("domain_id",did);}, {tok,did}); }
test("phone page renders panel", async({page})=>{
  // E2E: webphone family slow under vite dev + single worker; triple budget, asserts unchanged.
  test.slow();
  await login(page); await page.goto("/phone"); await expect(page.getByTestId("webphone-panel")).toBeVisible(); await expect(page.getByText(/Queue controls coming Phase2/)).toBeVisible(); });
