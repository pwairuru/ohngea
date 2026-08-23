import "@testing-library/jest-dom";
import * as React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { DataTable } from "./DataTable";
import { apiFetch } from "../lib/api";

vi.mock("../lib/api", () => ({ apiFetch: vi.fn() }));

const rows = [
  { extension_uuid: "u1", extension: "1001", effective_caller_id_name: "Alice" },
  { extension_uuid: "u2", extension: "1002", effective_caller_id_name: "Bob" },
];

describe("DataTable", () => {
  beforeEach(() => {
    (apiFetch as ReturnType<typeof vi.fn>).mockReset();
    (apiFetch as ReturnType<typeof vi.fn>).mockResolvedValue({ data: rows, total: 2 });
  });

  const cols = [
    { key: "extension", header: "Extension", sortable: true, className: "num" },
    { key: "effective_caller_id_name", header: "Name" },
  ];

  it("fetches and renders rows", async () => {
    render(<MemoryRouter><DataTable resource="/extensions" columns={cols} /></MemoryRouter>);
    expect((await screen.findAllByText("Alice"))[0]).toBeInTheDocument();
    expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining("/extensions?page=1&per_page=20"));
  });

  it("shows empty state when no rows", async () => {
    (apiFetch as ReturnType<typeof vi.fn>).mockResolvedValue({ data: [], total: 0 });
    render(<MemoryRouter><DataTable resource="/extensions" columns={cols} emptyTitle="No extensions yet" /></MemoryRouter>);
    const empties = await screen.findAllByTestId("empty-state");
    expect(empties[0]).toHaveTextContent("No extensions yet");
  });

  it("search debounces and adds q param", async () => {
    render(<MemoryRouter><DataTable resource="/extensions" columns={cols} /></MemoryRouter>);
    await screen.findAllByText("Alice");
    // initial call is without q
    expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining("q="));
    fireEvent.change(screen.getByPlaceholderText(/search/i), { target: { value: "alice" } });
    await waitFor(() => expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining("q=alice")), { timeout: 1500 });
  });

  it("sort header toggles order param", async () => {
    render(<MemoryRouter><DataTable resource="/extensions" columns={cols} /></MemoryRouter>);
    await screen.findAllByText("Alice");
    const btn = screen.getByRole("button", { name: /extension/i });
    fireEvent.click(btn);
    await waitFor(() => expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining("&sort=extension&order=asc")));
    fireEvent.click(btn);
    await waitFor(() => expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining("&sort=extension&order=desc")));
  });
});
