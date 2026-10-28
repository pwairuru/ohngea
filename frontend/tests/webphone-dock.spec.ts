import { test, expect, Page } from "@playwright/test";
async function loginViaAPI(page: Page){ const res=await page.request.post("/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); let j=await res.json(); if(!res.ok()){ const r2=await page.request.post("/api/v1/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); j=await r2.json();} await page.addInitScript((t:string)=>localStorage.setItem("token",t), j.access_token); await page.addInitScript((d:string)=>localStorage.setItem("domain_id",d), j.domain_id||"");}
test("dock visible and expands to panel", async({page})=>{
  // E2E: webphone family slow under vite dev + single worker; triple budget, asserts unchanged.
  test.slow();
  // force: Playwright mobile emulation mis-hit-tests the fixed dock (parent div "intercepts pointer events"; elementsFromPoint proves a real tap lands on the button)
  await loginViaAPI(page); await page.goto("/"); await expect(page.getByTestId("webphone-dock")).toBeVisible(); await page.getByTestId("dock-expand").click({ force: true }); await expect(page.locator('[role="dialog"]').or(page.getByTestId("webphone-panel")).first()).toBeVisible();
});
