import { test, expect, Page } from "@playwright/test";
async function loginViaAPI(page: Page){ const res=await page.request.post("http://localhost:8000/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); let j=await res.json(); if(!res.ok()){ const r2=await page.request.post("http://localhost:8000/api/v1/auth/login",{data:{username:"admin", password:"admin", domain:"test.example.com"}}); j=await r2.json();} await page.addInitScript((t:string)=>localStorage.setItem("token",t), j.access_token); await page.addInitScript((d:string)=>localStorage.setItem("domain_id",d), j.domain_id||"");}
test("dock visible and expands to panel", async({page})=>{
  await loginViaAPI(page); await page.goto("/"); await expect(page.getByTestId("webphone-dock")).toBeVisible(); await page.getByTestId("dock-expand").click(); await expect(page.locator('[role="dialog"]').or(page.getByTestId("webphone-panel"))).toBeVisible();
});
