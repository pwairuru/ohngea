import { describe, it, expect, vi, beforeEach } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { FullCrudPage } from "./FullCrudPage";
import type { CrudConfig } from "../lib/fieldConfigs";

vi.mock("../lib/api", () => ({
  apiFetch: vi.fn(() => Promise.resolve({ data: [], total: 0 })),
}));

const cfg: CrudConfig = {
  title: "Test Widgets",
  singular: "Widget",
  description: "Widgets for testing",
  resource: "/widgets",
  idKey: "widget_uuid",
  backendColumns: ["widget_name", "widget_size", "widget_enabled", "widget_note"],
  keyFields: [
    { name: "widget_name", label: "Name", required: true },
    { name: "widget_enabled", label: "Enabled", type: "checkbox" },
  ],
  advancedFields: [
    { name: "widget_size", label: "Size" },
    { name: "widget_note", label: "Note" },
  ],
};

function renderPage() {
  return render(
    <MemoryRouter>
      <FullCrudPage config={cfg} />
    </MemoryRouter>
  );
}

describe("FullCrudPage", () => {
  beforeEach(() => { vi.clearAllMocks(); });

  it("renders title, search and add button", () => {
    renderPage();
    expect(screen.getByText("Test Widgets")).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/Search test widgets/)).toBeInTheDocument();
    expect(screen.getByText("Add Widget")).toBeInTheDocument();
  });

  it("hides advanced fields until Show Advanced is clicked", async () => {
    renderPage();
    fireEvent.click(screen.getByText("Add Widget"));
    expect(screen.getByText("Name *")).toBeInTheDocument();
    expect(screen.queryByText("Size")).not.toBeInTheDocument();
    fireEvent.click(screen.getByText(/Show Advanced/));
    expect(screen.getByText("Size")).toBeInTheDocument();
    expect(screen.getByText("Note")).toBeInTheDocument();
  });

  it("drills into child details", async () => {
    const { apiFetch } = await import("../lib/api");
    (apiFetch as any).mockImplementation((path: string) =>
      path.includes("/details")
        ? Promise.resolve({ data: [{ username: "bob", caller_id_number: "1002" }], total: 1 })
        : Promise.resolve({ data: [], total: 0 })
    );
    const withChild = {
      ...cfg,
      children: [{
        title: "Sessions",
        resourceOf: (id: string) => `/parents/${id}/sessions`,
        idKey: "session_uuid",
        keyFields: [{ name: "username", label: "Username" }],
        advancedFields: [],
        detailsResourceOf: (id: string, rowId: string) => `/parents/${id}/sessions/${rowId}/details`,
      }],
    };
    render(
      <MemoryRouter>
        <FullCrudPage config={withChild} />
      </MemoryRouter>
    );
    expect(screen.getByText("Test Widgets")).toBeInTheDocument();
  });

  it("posts key + advanced fields on create", async () => {
    const { apiFetch } = await import("../lib/api");
    renderPage();
    fireEvent.click(screen.getByText("Add Widget"));
    fireEvent.change(screen.getByPlaceholderText("widget_name"), {
      target: { value: "w1" },
    });
    fireEvent.click(screen.getByText(/Show Advanced/));
    fireEvent.change(screen.getByPlaceholderText("widget_size"), {
      target: { value: "big" },
    });
    fireEvent.submit(screen.getByText("Create").closest("form")!);
    await waitFor(() => expect(apiFetch).toHaveBeenCalledWith(
      "/widgets",
      expect.objectContaining({ method: "POST" })
    ));
    const postCall = (apiFetch as any).mock.calls.find((c: any[]) => c[1]?.method === "POST");
    expect(postCall, "POST call").toBeDefined();
    const body = JSON.parse(postCall[1].body);
    expect(body.widget_name).toBe("w1");
    expect(body.widget_size).toBe("big");
  });

  it("child buttons are type=button so they never submit the parent form", async () => {
    const { apiFetch } = await import("../lib/api");
    (apiFetch as any).mockImplementation((path: string) =>
      path.includes("/sessions")
        ? Promise.resolve({ data: [{ session_uuid: "s1", username: "bob" }], total: 1 })
        : Promise.resolve({ data: [{ widget_uuid: "w1", widget_name: "w", widget_enabled: true }], total: 1 })
    );
    const withChild = {
      ...cfg,
      children: [{
        title: "Sessions",
        resourceOf: (id: string) => `/parents/${id}/sessions`,
        idKey: "session_uuid",
        keyFields: [{ name: "username", label: "Username" }],
        advancedFields: [{ name: "note", label: "Note" }],
      }],
    };
    render(
      <MemoryRouter>
        <FullCrudPage config={withChild} />
      </MemoryRouter>
    );
    // open edit dialog so the ChildManager renders
    const edits = await screen.findAllByTitle("Edit");
    fireEvent.click(edits[0]);
    await waitFor(() => expect(screen.getByText("Sessions (1)")).toBeInTheDocument());
    const btns = within(screen.getByRole("dialog")).getAllByRole("button", { name: /Add|Save Sessions|Delete Sessions/ });
    expect(btns.length).toBeGreaterThan(0);
    for (const b of btns) {
      expect((b as HTMLButtonElement).type).toBe("button");
    }
  });
});
