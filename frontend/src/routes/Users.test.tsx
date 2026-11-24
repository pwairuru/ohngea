import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { Users } from "./Users";

vi.mock("../lib/api", () => ({
  apiFetch: vi.fn((path: string, opts?: any) => {
    if (opts?.method === "POST" && path === "/users/import") {
      return Promise.resolve({ imported: 1, total: 1, errors: [], data: [] });
    }
    if (path.startsWith("/users")) return Promise.resolve({ data: [], total: 0 });
    if (path.startsWith("/groups")) return Promise.resolve({ data: [], total: 0 });
    if (path === "/auth/me") return Promise.resolve({ username: "admin", user_uuid: "u1", domain_id: "d1" });
    return Promise.resolve({ data: [], total: 0 });
  }),
}));

describe("Users", () => {
  it("imports users from CSV paste", async () => {
    const { apiFetch } = await import("../lib/api");
    render(
      <MemoryRouter>
        <Users />
      </MemoryRouter>
    );
    await waitFor(() => expect(screen.getByText("Users & Groups")).toBeInTheDocument());
    fireEvent.click(screen.getByText("Import"));
    expect(screen.getByText("Import Users (CSV)")).toBeInTheDocument();
    fireEvent.click(screen.getByText("Run Import"));
    await waitFor(() => expect(apiFetch).toHaveBeenCalledWith(
      "/users/import",
      expect.objectContaining({ method: "POST" })
    ));
    const postCall = (apiFetch as any).mock.calls.find((c: any[]) => c[0] === "/users/import");
    expect(JSON.parse(postCall[1].body).csv).toContain("username,email,password");
  });
});
