import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { WebrtcSettings } from "./WebrtcSettings";

vi.mock("../lib/api", () => ({
  apiFetch: vi.fn((path: string) => {
    if (path === "/auth/me") return Promise.resolve({ domain_id: "dom-1" });
    return Promise.resolve({ enabled: true, auth_mode: "both", wss_url: "wss://x", sip_domain: "x.com", ice_servers: [] });
  }),
}));

describe("WebrtcSettings", () => {
  it("loads domain settings into form", async () => {
    render(
      <MemoryRouter>
        <WebrtcSettings />
      </MemoryRouter>
    );
    expect(screen.getByText("WebRTC Settings")).toBeInTheDocument();
    await waitFor(() => expect(screen.getByDisplayValue("both")).toBeInTheDocument());
    expect(screen.getByDisplayValue("wss://x")).toBeInTheDocument();
    expect(screen.getByText("Save")).toBeInTheDocument();
  });
});
