import { describe, it, expect } from "vitest";
import * as fs from "fs";
import * as path from "path";
import { primaryNav, advancedNav, flatNav, routeMeta } from "./navConfig";

describe("navConfig", () => {
  it("has exactly 6 primary items", () => {
    expect(primaryNav.map((i) => i.to)).toEqual(["/", "/extensions", "/dialplans", "/gateways", "/devices", "/cdr"]);
  });
  it("has 5 advanced groups totalling 43 items", () => {
    expect(advancedNav.map((g) => g.label)).toEqual(["Call Routing", "Applications", "Network", "Status", "System"]);
    expect(advancedNav.reduce((n, g) => n + g.items.length, 0)).toBe(43);
  });
  it("every route exists in App.tsx Routes", () => {
    const app = fs.readFileSync(path.resolve(__dirname, "../App.tsx"), "utf8");
    const missing = flatNav.filter((i) => !app.includes(`path="${i.to}"`)).map((i) => i.to);
    expect(missing).toEqual([]);
  });
  it("routeMeta covers every item", () => {
    expect(Object.keys(routeMeta).length).toBe(flatNav.length);
  });
});
