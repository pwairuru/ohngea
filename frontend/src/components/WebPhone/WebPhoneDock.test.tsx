import "@testing-library/jest-dom/vitest";
import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
vi.mock("../../context/WebPhoneContext", ()=>({useWebPhone:()=>({registrationState:"registered", callState:"confirmed", duration:12, config:{auth_user:"1001"}, isMuted:false, mute:vi.fn(), hangup:vi.fn(), registrationError:null, currentSession:null, isHeld:false, connect:vi.fn(), disconnect:vi.fn(), call:vi.fn(), answer:vi.fn(), reject:vi.fn(), hold:vi.fn(), sendDTMF:vi.fn(), remoteAudioRef:{current:null}, history:[], contacts:[], addHistory:vi.fn(), clearHistory:vi.fn(), getHistoryKey:()=>null})}));
import { WebPhoneDock } from "./WebPhoneDock";
describe("WebPhoneDock", ()=>{
  it("shows timer when confirmed", ()=>{
    render(<WebPhoneDock onExpand={()=>{}}/>);
    expect(screen.getByTestId("webphone-dock")).toBeInTheDocument();
    expect(screen.getByTestId("webphone-dock-dot")).toHaveClass("bg-green-500");
    // timer appears twice (inline label + mono) — assert at least one
    expect(screen.getAllByText(/00:12/).length).toBeGreaterThanOrEqual(1);
  });
});
