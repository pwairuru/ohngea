import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { useEnabledApps } from "./apps";

describe("apps-nav", () => {
  beforeEach(() => {
    vi.stubGlobal("fetch", vi.fn());
    localStorage.setItem("domain_id", "test-domain");
    localStorage.setItem("token", "eyJhbGciOiJIUzI1NiJ9.eyJkb21haW5faWQiOiJ0ZXN0LWRvbWFpbiJ9.sig");
  });
  afterEach(() => {
    vi.unstubAllGlobals();
    localStorage.clear();
    vi.restoreAllMocks();
  });

  it("hides disabled app in Advanced drawer", async () => {
    const mockData = {
      data: [
        { app_name: "fifo", enabled: false },
        { app_name: "ivr_menus", enabled: true },
        { app_name: "conferences", enabled: true },
      ],
      total: 3,
    };
    (fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue({
      ok: true,
      status: 200,
      headers: { get: () => "application/json" },
      json: async () => mockData,
      text: async () => JSON.stringify(mockData),
    } as unknown as Response);

    const { result } = renderHook(() => useEnabledApps("test-domain"));

    // while loading, shows all (isEnabled true for any)
    expect(result.current.loading).toBe(true);
    expect(result.current.isEnabled("fifo")).toBe(true);

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.isEnabled("fifo")).toBe(false);
    expect(result.current.isEnabled("ivr_menus")).toBe(true);
    expect(result.current.isEnabled("conferences")).toBe(true);
    expect(result.current.enabled.has("fifo")).toBe(false);
    expect(result.current.enabled.has("ivr_menus")).toBe(true);
  });

  it("shows all until fetch completes", async () => {
    let resolve!: (v: Response) => void;
    const pending = new Promise<Response>((r) => (resolve = r));
    (fetch as unknown as ReturnType<typeof vi.fn>).mockReturnValue(pending);

    const { result } = renderHook(() => useEnabledApps("test-domain"));
    expect(result.current.loading).toBe(true);
    // isEnabled true while loading
    expect(result.current.isEnabled("any_app")).toBe(true);

    const mockData = { data: [{ app_name: "any_app", enabled: false }], total: 1 };
    resolve({
      ok: true,
      status: 200,
      headers: { get: () => "application/json" },
      json: async () => mockData,
      text: async () => JSON.stringify(mockData),
    } as unknown as Response);

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.isEnabled("any_app")).toBe(false);
  });
});
