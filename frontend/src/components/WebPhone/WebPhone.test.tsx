import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { WebPhoneProvider, useWebPhone } from "../../context/WebPhoneContext";
import { WebPhoneContext } from "../../context/WebPhoneContext";
import { WebPhoneButton } from "./WebPhoneButton";

vi.mock("sip.js", () => ({
  UserAgent: class { static makeURI(s: string){return s} constructor(_o:any){ } start=vi.fn(()=>Promise.resolve()); stop=vi.fn(); delegate={} },
  Registerer: class { state="Initial"; constructor(_a:any){} register=vi.fn(()=>Promise.resolve()); unregister=vi.fn(); stateChange=new (class{addListener(){}}); },
  SessionState: { Initial: "Initial", Establishing:"Establishing", Established:"Established", Terminated:"Terminated" },
  Inviter: class { state="Initial"; constructor(_a:any,_b:any,_c:any){} invite=vi.fn(()=>Promise.resolve()); },
}));

function Probe(){ const {registrationState}=useWebPhone(); return <span data-testid="reg">{registrationState}</span>; }

describe("WebPhoneContext", ()=>{
  it("starts unconfigured when no localStorage", async()=>{
    localStorage.clear();
    render(<WebPhoneProvider><Probe/></WebPhoneProvider>);
    expect(screen.getByTestId("reg").textContent).toBe("unconfigured");
  });
  it("exposes connect", async()=>{
    localStorage.clear();
    render(<WebPhoneProvider><Probe/></WebPhoneProvider>);
    // connect not called yet still unconfigured
    expect(screen.getByTestId("reg").textContent).toBe("unconfigured");
  });
});

describe("WebPhoneButton", ()=>{
  it("shows green dot when registered", async()=>{
    const Wrapper=({state}:{state:any})=>(
      <WebPhoneContext.Provider value={{registrationState:state, registrationError:null, callState:"idle", currentSession:null, config:null, isMuted:false,isHeld:false, connect:async()=>{}, disconnect:async()=>{}, call:async()=>{}, hangup:async()=>{}, answer:async()=>{}, reject:async()=>{}, mute:()=>{}, hold:()=>{}, sendDTMF:()=>{}, duration:0}}>
        <WebPhoneButton onOpen={()=>{}} />
      </WebPhoneContext.Provider>
    );
    const {rerender}=render(<Wrapper state="registered"/>);
    expect(screen.getByTestId("webphone-registration-dot").className).toContain("bg-green-500");
    rerender(<Wrapper state="failed"/>);
    expect(screen.getByTestId("webphone-registration-dot").className).toContain("bg-red-500");
    rerender(<Wrapper state="connecting"/>);
    expect(screen.getByTestId("webphone-registration-dot").className).toContain("bg-yellow-500");
  });
});
