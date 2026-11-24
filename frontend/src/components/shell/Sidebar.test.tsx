import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { Sidebar } from "./Sidebar";

vi.mock("../../lib/api", () => ({
  apiFetch: vi.fn((path: string) => {
    if (path.startsWith("/menu_items")) {
      return Promise.resolve({
        data: [
          { menu_item_uuid: "m1", parent_uuid: null, title: "Custom Group", path: "", icon: "", item_order: 0, groups: "", enabled: true },
          { menu_item_uuid: "m2", parent_uuid: "m1", title: "My Page", path: "/my-page", icon: "Phone", item_order: 0, groups: "", enabled: true },
        ],
        total: 2,
      });
    }
    return Promise.resolve({ data: [], total: 0 });
  }),
}));

describe("Sidebar dynamic menu", () => {
  it("renders menu_items tree when present", async () => {
    render(<MemoryRouter><Sidebar /></MemoryRouter>);
    await waitFor(() => expect(screen.getByText("Custom Group")).toBeInTheDocument());
    fireEvent.click(screen.getByText("Custom Group"));
    expect(screen.getByText("My Page")).toBeInTheDocument();
  });

  it("falls back to static nav when empty", async () => {
    const { apiFetch } = await import("../../lib/api");
    (apiFetch as any).mockResolvedValueOnce({ data: [], total: 0 });
    render(<MemoryRouter><Sidebar /></MemoryRouter>);
    await waitFor(() => expect(screen.getByText("Dashboard")).toBeInTheDocument());
  });
});
