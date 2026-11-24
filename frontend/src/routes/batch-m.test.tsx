import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";

vi.mock("../lib/api", () => ({
  apiFetch: vi.fn((path: string, opts?: any) => {
    if (opts?.method === "POST") return Promise.resolve({ result: "ok" });
    if (path.startsWith("/voicemails/") && path.includes("/messages")) {
      return Promise.resolve({ data: [{ voicemail_message_uuid: "m1", caller_id_number: "1001", message_status: "saved" }], total: 1 });
    }
    if (path.startsWith("/voicemails") && !path.includes("/messages") && !path.includes("/options")) {
      return Promise.resolve({ data: [{ voicemail_uuid: "v1", voicemail_id: "1001", enabled: true }], total: 1 });
    }
    if (path.includes("/options")) return Promise.resolve({ data: [], total: 0 });
    if (path.startsWith("/ivr_menus")) {
      return Promise.resolve({ data: [{ ivr_menu_uuid: "i1", name: "main", extension: "5000", enabled: true }], total: 1 });
    }
    if (path.includes("/agents") || path.includes("/tiers")) return Promise.resolve({ data: [], total: 0 });
    if (path.startsWith("/call_centers")) {
      return Promise.resolve({ data: [{ call_center_queue_uuid: "q1", queue_name: "sales", queue_extension: "6000", queue_enabled: true }], total: 1 });
    }
    if (path.startsWith("/fax")) {
      return Promise.resolve({ data: [{ fax_uuid: "f1", fax_extension: "7000", fax_name: "fax1", enabled: true }], total: 1 });
    }
    return Promise.resolve({ data: [], total: 0 });
  }),
}));

vi.mock("../components/ui/confirm", () => ({
  useConfirm: () => async () => true,
}));

describe("batch M messaging/media", () => {
  it("Voicemails edit manages messages and options", async () => {
    const { Voicemails } = await import("./Voicemails");
    render(<MemoryRouter><Voicemails /></MemoryRouter>);
    await waitFor(() => expect(screen.getAllByText("Edit").length).toBeGreaterThanOrEqual(1));
    fireEvent.click(screen.getAllByText("Edit")[0]);
    await waitFor(() => expect(screen.getByText("Messages (1)")).toBeInTheDocument());
    expect(screen.getByText("Options (0)")).toBeInTheDocument();
    expect(screen.getByText("saved")).toBeInTheDocument();
  });

  it("IvrMenus edit manages options", async () => {
    const { IvrMenus } = await import("./IvrMenus");
    render(<MemoryRouter><IvrMenus /></MemoryRouter>);
    await waitFor(() => expect(screen.getAllByText("Edit").length).toBeGreaterThanOrEqual(1));
    fireEvent.click(screen.getAllByText("Edit")[0]);
    await waitFor(() => expect(screen.getByText("Options (0)")).toBeInTheDocument());
  });

  it("CallCenters edit manages agents and tiers", async () => {
    const { CallCenters } = await import("./CallCenters");
    render(<MemoryRouter><CallCenters /></MemoryRouter>);
    await waitFor(() => expect(screen.getAllByText("Edit").length).toBeGreaterThanOrEqual(1));
    fireEvent.click(screen.getAllByText("Edit")[0]);
    await waitFor(() => expect(screen.getByText("Agents (0)")).toBeInTheDocument());
    expect(screen.getByText("Tiers (0)")).toBeInTheDocument();
  });

  it("Fax row offers Send with number dialog", async () => {
    const { Fax } = await import("./Fax");
    render(<MemoryRouter><Fax /></MemoryRouter>);
    await waitFor(() => expect(screen.getAllByText("Send").length).toBeGreaterThanOrEqual(1));
    fireEvent.click(screen.getAllByText("Send")[0]);
    expect(screen.getByText("Destination number *")).toBeInTheDocument();
  });

  it("Fax row offers Upload dialog", async () => {
    const { Fax } = await import("./Fax");
    render(<MemoryRouter><Fax /></MemoryRouter>);
    await waitFor(() => expect(screen.getAllByText("Upload").length).toBeGreaterThanOrEqual(1));
    fireEvent.click(screen.getAllByText("Upload")[0]);
    expect(screen.getByText("File *")).toBeInTheDocument();
  });

  it("Voicemail messages offer Play", async () => {
    const { Voicemails } = await import("./Voicemails");
    render(<MemoryRouter><Voicemails /></MemoryRouter>);
    await waitFor(() => expect(screen.getAllByText("Edit").length).toBeGreaterThanOrEqual(1));
    fireEvent.click(screen.getAllByText("Edit")[0]);
    await waitFor(() => expect(screen.getAllByText("Play").length).toBeGreaterThanOrEqual(1));
  });

  it("Cdr offers Export CSV", async () => {
    const { Cdr } = await import("./Cdr");
    render(<MemoryRouter><Cdr /></MemoryRouter>);
    expect(screen.getByText("Export CSV")).toBeInTheDocument();
  });
});
