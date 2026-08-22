import { describe, it, expect, vi, beforeEach } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";

// polyfill ResizeObserver for recharts
class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}
(globalThis as any).ResizeObserver = MockResizeObserver;

import { Dashboard } from "./Dashboard";

const mockNavigate = vi.fn();
vi.mock("react-router-dom", async () => {
  const actual: any = await vi.importActual("react-router-dom");
  return { ...actual, useNavigate: () => mockNavigate };
});

vi.mock("../lib/api", () => ({
  apiFetch: vi.fn((path: string) => {
    if (path.includes("/analytics/summary")) return Promise.resolve({ total_calls: 10, today_calls: 2, avg_duration: 30 });
    if (path.includes("/analytics/daily")) return Promise.resolve({ data: [{ day: "2026-08-14", count: 5 }] });
    if (path.includes("/analytics/by_direction")) return Promise.resolve({ data: [{ direction: "inbound", count: 5 }, { direction: "outbound", count: 3 }] });
    if (path.includes("/analytics/by_hangup")) return Promise.resolve({ data: [{ hangup_cause: "NORMAL_CLEARING", count: 8 }] });
    if (path.includes("/analytics/top_extensions")) return Promise.resolve({ data: [{ extension: "1001", count: 4 }] });
    if (path.includes("/analytics/hourly")) return Promise.resolve({ data: [{ hour: new Date().toISOString(), count: 2 }] });
    if (path.includes("/analytics/system_counts")) return Promise.resolve({ extensions: 5, devices: 2, gateways: 1, sip_profiles: 1, enabled_extensions: 4, disabled_extensions: 1, voicemails: 0 });
    if (path.includes("/analytics/answer_rate")) return Promise.resolve({ answered: 8, missed: 2, total: 10, answer_rate: 80, avg_billsec: 25 });
    if (path.includes("/sip_profiles/status")) return Promise.resolve({ connected: true, profiles: [{ running: true }, { running: false }] });
    if (path.includes("/freeswitch/status")) return Promise.resolve({ connected: true, latency_ms: 5 });
    return Promise.resolve({});
  }),
}));

describe("Dashboard", () => {
  beforeEach(() => mockNavigate.mockClear());
  it("renders heading and FS status and charts", async () => {
    render(<MemoryRouter><Dashboard /></MemoryRouter>);
    expect(await screen.findByText("Dashboard")).toBeInTheDocument();
    // quick add button navigates
    const btn = await screen.findByTestId("dashboard-add-extension");
    expect(btn).toBeInTheDocument();
    fireEvent.click(btn);
    expect(mockNavigate).toHaveBeenCalledWith("/extensions");
  });
  it("renders key chart titles", async () => {
    render(<MemoryRouter><Dashboard /></MemoryRouter>);
    expect(await screen.findByText("CDR Daily (7 days)")).toBeInTheDocument();
    expect(await screen.findByText("Calls by Direction")).toBeInTheDocument();
    expect(await screen.findByText("Top Extensions")).toBeInTheDocument();
    expect(await screen.findByText("System Overview")).toBeInTheDocument();
    expect(await screen.findByText("Hourly Today")).toBeInTheDocument();
  });
});
