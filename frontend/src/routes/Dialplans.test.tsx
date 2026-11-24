import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { Dialplans } from "./Dialplans";

vi.mock("../lib/api", () => ({
  apiFetch: vi.fn(() => Promise.resolve({ data: [], total: 0 })),
}));

vi.mock("../components/ui/confirm", () => ({
  useConfirm: () => async () => true,
}));

describe("Dialplans", () => {
  it("full PHP-parity form renders on Add", async () => {
    render(
      <MemoryRouter>
        <Dialplans />
      </MemoryRouter>
    );
    fireEvent.click(screen.getByText("Add Dialplan"));
    await waitFor(() => expect(screen.getByText("Destination regex")).toBeInTheDocument());
    for (const label of ["Number", "Order", "Description", "Hostname", "Continue", "Destination regex"]) {
      expect(screen.getByText(label, { exact: false })).toBeInTheDocument();
    }
  });
});
