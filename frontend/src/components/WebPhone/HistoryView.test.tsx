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
  it("shows contact name when peerName missing", ()=>{
    render(<HistoryView history={[{id:"2", direction:"in", peer:"1001", peerName:null, startedAt:new Date().toISOString(), duration:5, disposition:"answered", domain_id:"d", user_uuid:"u"}]} contacts={[{id:"ext-1001", primary:"1001", name:"Alice", favorite:false, domain_id:"d"}]} onRedial={()=>{}} onClear={()=>{}}/>);
    expect(screen.getByText(/Alice/)).toBeInTheDocument();
  });
});
