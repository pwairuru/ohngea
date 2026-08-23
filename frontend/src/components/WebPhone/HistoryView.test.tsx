import "@testing-library/jest-dom/vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { HistoryView } from "./HistoryView";
describe("HistoryView", ()=>{
  it("calls redial", ()=>{
    const onRedial=vi.fn(); render(<HistoryView history={[{id:"1", direction:"out", peer:"1002", peerName:"B", startedAt:new Date().toISOString(), duration:5, disposition:"answered", domain_id:"d", user_uuid:"u"}]} onRedial={onRedial} onClear={()=>{}}/>);
    fireEvent.click(screen.getByTestId("history-redial-1002")); expect(onRedial).toHaveBeenCalledWith("1002");
  });
  it("shows empty", ()=>{ render(<HistoryView history={[]} onRedial={()=>{}} onClear={()=>{}}/>); expect(screen.getByText(/No recent calls/)).toBeInTheDocument(); });
});
