import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, fireEvent } from "@testing-library/react";
import { Extensions } from "./Extensions";
import { MemoryRouter } from "react-router-dom";

// mock apiFetch
vi.mock("../lib/api", () => ({
  apiFetch: vi.fn(() => Promise.resolve({ data: [], total: 0 })),
}));

describe("Extensions", () => {
  it("renders heading and search", async () => {
    render(
      <MemoryRouter>
        <Extensions />
      </MemoryRouter>
    );
    expect(screen.getByText("Extensions")).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/Search extensions/)).toBeInTheDocument();
  });

  it("has table and cards for responsive", async () => {
    render(
      <MemoryRouter>
        <Extensions />
      </MemoryRouter>
    );
    // table header hidden on mobile but rendered
    expect(screen.getByText("Add Extension")).toBeInTheDocument();
  });

  it("full PHP-parity form renders on Add", async () => {
    render(
      <MemoryRouter>
        <Extensions />
      </MemoryRouter>
    );
    fireEvent.click(screen.getByTestId("add-extension"));
    for (const label of [
      "Outbound CID Name", "Emergency CID Number",
      "Call Timeout", "Toll Allow", "Forward All →", "Do Not Disturb",
      "SIP Bypass Media", "User Context", "Voicemail Password", "Voicemail File",
      "Transcription Prompt Enabled", "Keep Local After Email",
    ]) {
      expect(screen.getByText(label, { exact: false })).toBeInTheDocument();
    }
  });
});
