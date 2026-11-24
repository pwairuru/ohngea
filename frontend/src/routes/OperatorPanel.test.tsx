import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { OperatorPanel } from "./OperatorPanel";

vi.mock("../lib/api", () => ({
  apiFetch: vi.fn((path: string) => {
    if (path.startsWith("/operator_panel/calls")) {
      return Promise.resolve({ data: [{ uuid: "call-1", caller_id_number: "1001", destination_number: "1002" }], total: 1 });
    }
    return Promise.resolve({ domain_id: "d1", extensions_total: 5, active: [], queues: [] });
  }),
}));

describe("OperatorPanel", () => {
  it("renders originate form and per-call actions", async () => {
    render(
      <MemoryRouter>
        <OperatorPanel />
      </MemoryRouter>
    );
    expect(screen.getByText("Operator Panel")).toBeInTheDocument();
    await waitFor(() => expect(screen.getByText("1001")).toBeInTheDocument());
    expect(screen.getByPlaceholderText("Destination")).toBeInTheDocument();
    expect(screen.getAllByText("Hangup").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Eavesdrop").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Coach").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Record").length).toBeGreaterThanOrEqual(1);
  });
});
