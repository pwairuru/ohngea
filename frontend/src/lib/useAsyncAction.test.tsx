import { describe, it, expect } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useAsyncAction } from "./useAsyncAction";

describe("useAsyncAction", () => {
  it("sets pending during run and clears after", async () => {
    let resolveFn: () => void = () => {};
    const { result } = renderHook(() => useAsyncAction(() => new Promise<void>((r) => (resolveFn = r))));
    expect(result.current.pending).toBe(false);
    act(() => { result.current.run(); });
    expect(result.current.pending).toBe(true);
    resolveFn();
    await waitFor(() => expect(result.current.pending).toBe(false));
  });
  it("clears pending even when fn rejects", async () => {
    const { result } = renderHook(() => useAsyncAction(() => Promise.reject(new Error("boom"))));
    await act(async () => { await result.current.run().catch(() => {}); });
    expect(result.current.pending).toBe(false);
  });
});
