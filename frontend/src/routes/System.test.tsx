import { describe, it, expect, vi, beforeEach } from "vitest";
import "@testing-library/jest-dom";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";

// polyfill ResizeObserver for recharts
class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}
(globalThis as any).ResizeObserver = MockResizeObserver;

import { apiFetch } from "../lib/api";

vi.mock("../lib/api", () => ({
  apiFetch: vi.fn(),
}));

import { System } from "./System";

const triple = { hostname: "test", load_avg: [0.5, 0.4, 0.3], cpu: { cores: 4, total_percent: 10, per_core: [10, 10, 10, 10] }, memory: { total_bytes: 1000, used_bytes: 500, available_bytes: 500, percent: 50 }, disk: [] };
const tripleMetrics = { hostname: "test", load_avg: [0.5, 0.4, 0.3], cpu_total: 10, cpu_cores: 4, cpu_per_core: [10, 10, 10, 10], memory: { total_bytes: 1000, used_bytes: 500, available_bytes: 500, percent: 50 }, disks: [] };

describe("System load_avg contract", () => {
  beforeEach(() => {
    vi.mocked(apiFetch).mockImplementation((path: string) => {
      if (path.includes("/system/info")) return Promise.resolve(triple);
      if (path.includes("/system/metrics")) return Promise.resolve(tripleMetrics);
      return Promise.resolve({});
    });
  });
  it("renders triple array without crashing", async () => {
    render(<MemoryRouter><System /></MemoryRouter>);
    expect(await screen.findByText("System Status")).toBeInTheDocument();
    // 1m/5m/15m chips render
    expect(await screen.findByText("0.50")).toBeInTheDocument();
  });
  it("tolerates legacy bare-number load_avg without crashing", async () => {
    vi.mocked(apiFetch).mockImplementation((path: string) => {
      if (path.includes("/system/info")) return Promise.resolve({ ...triple, load_avg: 1.04 });
      if (path.includes("/system/metrics")) return Promise.resolve({ ...tripleMetrics, load_avg: 1.04 });
      return Promise.resolve({});
    });
    render(<MemoryRouter><System /></MemoryRouter>);
    expect(await screen.findByText("System Status")).toBeInTheDocument();
    // legacy number normalized to single chip
    expect(await screen.findByText("1.04")).toBeInTheDocument();
  });
});
