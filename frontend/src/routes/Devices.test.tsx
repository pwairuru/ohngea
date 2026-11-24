import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { Devices } from "./Devices";

vi.mock("../lib/api", () => ({
  apiFetch: vi.fn((path: string) => {
    if (path.startsWith("/devices/")) return Promise.resolve({ data: [], total: 0 });
    return Promise.resolve({
      data: [{ device_uuid: "dev-1", mac: "00:11:22:33:44:55", vendor: "poly", model: "VVX250", label: "", enabled: true }],
      total: 1,
    });
  }),
}));

vi.mock("../components/ui/confirm", () => ({
  useConfirm: () => async () => true,
}));

describe("Devices", () => {
  it("edit dialog manages lines, keys and settings", async () => {
    render(
      <MemoryRouter>
        <Devices />
      </MemoryRouter>
    );
    await waitFor(() => expect(screen.getAllByText("00:11:22:33:44:55").length).toBeGreaterThanOrEqual(1));
    fireEvent.click(screen.getAllByText("Edit")[0]);
    await waitFor(() => expect(screen.getByText("Lines (0)")).toBeInTheDocument());
    expect(screen.getByText("Keys (0)")).toBeInTheDocument();
    expect(screen.getByText("Settings (0)")).toBeInTheDocument();
    // PHP-parity fields present in the form
    for (const label of ["Template", "Location", "Serial Number", "Firmware Version", "Description", "Username", "Password", "Profile UUID"]) {
      expect(screen.getByText(label, { exact: false })).toBeInTheDocument();
    }
  });
});
