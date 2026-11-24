import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { Software } from "./Software";
import { Databases } from "./Databases";
import { Notifications } from "./Notifications";

vi.mock("../lib/api", () => ({
  apiFetch: vi.fn((path: string) => {
    if (path.includes("/software")) return Promise.resolve({ data: [{ software_uuid: "s1", software_name: "fs", software_version: "1.10.12" }], total: 1 });
    if (path.includes("/databases")) return Promise.resolve({ data: [{ database_uuid: "d1", database_name: "pbx", database_type: "pgsql" }], total: 1 });
    return Promise.resolve({ data: [{ notification_uuid: "n1", project_notifications: "hello" }], total: 1 });
  }),
}));

describe("Software/Databases/Notifications pages", () => {
  it("software page lists versions", async () => {
    render(<MemoryRouter><Software /></MemoryRouter>);
    await waitFor(() => expect(screen.getAllByText("1.10.12").length).toBeGreaterThan(0));
  });
  it("databases page lists connections", async () => {
    render(<MemoryRouter><Databases /></MemoryRouter>);
    await waitFor(() => expect(screen.getAllByText("pbx").length).toBeGreaterThan(0));
  });
  it("notifications page shows PHP-parity blob field", async () => {
    render(<MemoryRouter><Notifications /></MemoryRouter>);
    await waitFor(() => expect(screen.getAllByText("hello").length).toBeGreaterThan(0));
  });
});
