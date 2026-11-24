import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { UserSettings } from "./UserSettings";

vi.mock("../lib/api", () => ({
  apiFetch: vi.fn((path: string) => {
    if (path === "/auth/me") return Promise.resolve({ user_uuid: "u1", username: "alice" });
    return Promise.resolve({ data: [{ user_setting_uuid: "s1", category: "profile", name: "language", value: "en", enabled: true }], total: 1 });
  }),
}));

describe("UserSettings", () => {
  it("loads own settings", async () => {
    render(<MemoryRouter><UserSettings /></MemoryRouter>);
    expect(screen.getByText("My Settings")).toBeInTheDocument();
    await waitFor(() => expect(screen.getAllByText("language").length).toBeGreaterThanOrEqual(1));
    expect(screen.getByText("Add Setting")).toBeInTheDocument();
  });
});
