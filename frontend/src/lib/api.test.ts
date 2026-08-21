import { describe, it, expect } from "vitest";
import { API_BASE } from "./api";

describe("api", () => {
  it("has correct base", () => {
    expect(API_BASE).toBe("/api/v1");
  });
});
