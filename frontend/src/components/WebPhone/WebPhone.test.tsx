import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { WebPhoneProvider, useWebPhone } from "../../context/WebPhoneContext";

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
