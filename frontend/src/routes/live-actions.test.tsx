import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";

vi.mock("../lib/api", () => ({
  apiFetch: vi.fn((path: string, opts?: any) => {
    if (opts?.method === "POST") return Promise.resolve({ result: "ok", send: { attempted: 1 } });
    if (path.includes("/members") && !path.includes("/mute")) {
      return Promise.resolve({ data: [{ member_id: "1", caller_id_number: "1001", muted: false }], total: 1 });
    }
    if (path.includes("/callers")) {
      return Promise.resolve({ data: [{ uuid: "u1", caller_id_number: "1002", wait_time: 5 }], total: 1 });
    }
    if (path.startsWith("/active_calls")) {
      return Promise.resolve({ data: [{ call_uuid: "c1", direction: "inbound", cid_num: "1001", dest: "1002", state: "ACTIVE" }], total: 1 });
    }
    if (path.startsWith("/active_conferences")) {
      return Promise.resolve({ data: [{ conference_name: "3001", member_count: 1 }], total: 1 });
    }
    if (path.startsWith("/fifo_list")) {
      return Promise.resolve({ data: [{ fifo_name: "support" }], total: 1 });
    }
    if (path.startsWith("/call_broadcasts")) {
      return Promise.resolve({ data: [{ call_broadcast_uuid: "b1", broadcast_name: "camp1", broadcast_enabled: true }], total: 1 });
    }
    return Promise.resolve({ data: [], total: 0 });
  }),
}));

describe("live telephony actions", () => {
  it("ActiveCalls hangs up a call", async () => {
    const { ActiveCalls } = await import("./ActiveCalls");
    const { apiFetch } = await import("../lib/api");
    render(<MemoryRouter><ActiveCalls /></MemoryRouter>);
    await waitFor(() => expect(screen.getAllByText("Hangup").length).toBeGreaterThanOrEqual(1));
    window.confirm = vi.fn(() => true);
    fireEvent.click(screen.getAllByText("Hangup")[0]);
    await waitFor(() => expect(apiFetch).toHaveBeenCalledWith("/active_calls/c1/hangup", expect.objectContaining({ method: "POST" })));
  });

  it("ActiveConferences drills into members with controls", async () => {
    const { ActiveConferences } = await import("./ActiveConferences");
    render(<MemoryRouter><ActiveConferences /></MemoryRouter>);
    await waitFor(() => expect(screen.getAllByText("Members").length).toBeGreaterThanOrEqual(1));
    fireEvent.click(screen.getAllByText("Members")[0]);
    await waitFor(() => expect(screen.getByText(/Members of 3001/)).toBeInTheDocument());
    expect(screen.getAllByText("mute").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("kick").length).toBeGreaterThanOrEqual(1);
  });

  it("FifoList drills into callers with remove", async () => {
    const { FifoList } = await import("./FifoList");
    render(<MemoryRouter><FifoList /></MemoryRouter>);
    await waitFor(() => expect(screen.getAllByText("Callers").length).toBeGreaterThanOrEqual(1));
    fireEvent.click(screen.getAllByText("Callers")[0]);
    await waitFor(() => expect(screen.getByText(/Callers in support/)).toBeInTheDocument());
    expect(screen.getByText("Remove")).toBeInTheDocument();
  });

  it("CallBroadcasts offers Send and Stop", async () => {
    const { CallBroadcasts } = await import("./CallBroadcasts");
    render(<MemoryRouter><CallBroadcasts /></MemoryRouter>);
    await waitFor(() => expect(screen.getAllByText("Send").length).toBeGreaterThanOrEqual(1));
    expect(screen.getAllByText("Stop").length).toBeGreaterThanOrEqual(1);
  });
});
