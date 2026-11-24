import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { LiveTable } from "./LiveTable";

vi.mock("../lib/api", () => ({
  apiFetch: vi.fn(() => Promise.resolve({
    data: [{ call_uuid: "abc", direction: "inbound", caller_id_number: "1001", duration: 42 }],
    total: 1,
  })),
}));

describe("LiveTable", () => {
  it("renders live rows with auto-refresh note", async () => {
    render(
      <MemoryRouter>
        <LiveTable title="Active Calls" description="Live channels" resource="/active_calls" refreshSecs={0} />
      </MemoryRouter>
    );
    expect(screen.getByText("Active Calls")).toBeInTheDocument();
    await waitFor(() => expect(screen.getAllByText("1001").length).toBeGreaterThanOrEqual(1));
    expect(screen.getByText(/auto-refresh/)).toBeInTheDocument();
  });
});
