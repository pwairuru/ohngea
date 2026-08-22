import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { WebPhoneProvider, useWebPhone } from "../../context/WebPhoneContext";
import { WebPhoneContext } from "../../context/WebPhoneContext";
import { WebPhoneButton } from "./WebPhoneButton";
import { WebPhonePopup } from "./WebPhonePopup";
import { apiFetch } from "../../lib/api";

vi.mock("sip.js", () => ({
  UserAgent: class { static makeURI(s: string){return s} constructor(_o:any){ } start=vi.fn(()=>Promise.resolve()); stop=vi.fn(); delegate={} },
  Registerer: class { state="Initial"; constructor(_a:any){} register=vi.fn(()=>Promise.resolve()); unregister=vi.fn(); stateChange=new (class{addListener(){}}); },
  SessionState: { Initial: "Initial", Establishing:"Establishing", Established:"Established", Terminated:"Terminated" },
  Inviter: class { state="Initial"; constructor(_a:any,_b:any,_c:any){} invite=vi.fn(()=>Promise.resolve()); },
}));

vi.mock("../../lib/api", async () => {
  const actual = await vi.importActual("../../lib/api") as any;
  return {
    ...actual,
    apiFetch: vi.fn((path: string, opts?: any) => {
      if (path.includes("/webrtc_settings")) return Promise.resolve({ enabled: true, auth_mode: "both", wss_url: "wss://pbx.example.com/wss", ice_servers: [] });
      if (path.includes("/extensions")) return Promise.resolve({ data: [{ extension_uuid: "u1", extension: "1001", effective_caller_id_name: "Alice" }, { extension_uuid: "u2", extension: "1002", effective_caller_id_name: "Bob" }] });
      if (path === "/webrtc/credentials") return Promise.resolve({ sip_uri: "sip:1001@example.com", wss_url: "wss://pbx.example.com/wss", auth_user: "1001", password: "secret" });
      return Promise.resolve({});
    }),
  };
});

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

describe("WebPhonePopup", ()=>{
  beforeEach(()=>{
    localStorage.clear();
    const payload = { domain_id: "test-domain", sub: "test-user" };
    const token = `h.${btoa(JSON.stringify(payload))}.s`;
    localStorage.setItem("token", token);
    localStorage.setItem("domain_id", "test-domain");
    vi.clearAllMocks();
  });

  it("shows config form when unconfigured and dialpad when configured", async()=>{
    const UnconfiguredWrapper = ()=>(
      <WebPhoneContext.Provider value={{registrationState:"unconfigured", registrationError:null, callState:"idle", currentSession:null, config:null, isMuted:false,isHeld:false, connect:vi.fn(async()=>{}), disconnect:vi.fn(async()=>{}), call:vi.fn(async()=>{}), hangup:vi.fn(async()=>{}), answer:vi.fn(async()=>{}), reject:vi.fn(async()=>{}), mute:vi.fn(), hold:vi.fn(), sendDTMF:vi.fn(), duration:0}}>
        <WebPhonePopup open onOpenChange={()=>{}} />
      </WebPhoneContext.Provider>
    );
    const {unmount} = render(<UnconfiguredWrapper/>);
    expect(await screen.findByTestId("webphone-config-form")).toBeInTheDocument();
    expect(screen.queryByTestId("webphone-dialpad")).not.toBeInTheDocument();
    unmount();

    const ConfiguredWrapper = ()=>(
      <WebPhoneContext.Provider value={{registrationState:"registered", registrationError:null, callState:"idle", currentSession:null, config:{sip_uri:"sip:1001@example.com", wss_url:"wss://pbx.example.com/wss", auth_user:"1001", password:"secret"}, isMuted:false,isHeld:false, connect:vi.fn(async()=>{}), disconnect:vi.fn(async()=>{}), call:vi.fn(async()=>{}), hangup:vi.fn(async()=>{}), answer:vi.fn(async()=>{}), reject:vi.fn(async()=>{}), mute:vi.fn(), hold:vi.fn(), sendDTMF:vi.fn(), duration:0}}>
        <WebPhonePopup open onOpenChange={()=>{}} />
      </WebPhoneContext.Provider>
    );
    render(<ConfiguredWrapper/>);
    expect(await screen.findByTestId("webphone-dialpad")).toBeInTheDocument();
    expect(screen.queryByTestId("webphone-config-form")).not.toBeInTheDocument();
  });

  it("config form validates wss_url", async()=>{
    const Wrapper = ()=>(
      <WebPhoneContext.Provider value={{registrationState:"unconfigured", registrationError:null, callState:"idle", currentSession:null, config:null, isMuted:false,isHeld:false, connect:vi.fn(async()=>{}), disconnect:vi.fn(async()=>{}), call:vi.fn(async()=>{}), hangup:vi.fn(async()=>{}), answer:vi.fn(async()=>{}), reject:vi.fn(async()=>{}), mute:vi.fn(), hold:vi.fn(), sendDTMF:vi.fn(), duration:0}}>
        <WebPhonePopup open onOpenChange={()=>{}} />
      </WebPhoneContext.Provider>
    );
    render(<Wrapper/>);
    await screen.findByTestId("webphone-config-form");
    // switch to Custom SIP to show wss input (auth_mode both -> button)
    const customBtn = await screen.findByText("Custom SIP");
    fireEvent.click(customBtn);
    // find wss input by placeholder
    const wssInput = screen.getByPlaceholderText("wss://pbx.example.com/wss");
    fireEvent.change(wssInput, {target:{value:"http://bad.example.com"}});
    const saveBtn = screen.getByTestId("save-register");
    fireEvent.click(saveBtn);
    expect(await screen.findByText(/WSS URL must start with wss:\/\//i)).toBeInTheDocument();
  });

  it("shows incoming banner with Answer/Reject and dialpad grid/call controls", async()=>{
    const answer = vi.fn(async()=>{}); const reject = vi.fn(async()=>{}); const callFn = vi.fn(async()=>{}); const hangup = vi.fn(async()=>{}); const muteFn = vi.fn();
    const Wrapper = ()=>(
      <WebPhoneContext.Provider value={{registrationState:"registered", registrationError:null, callState:"incoming", currentSession:{} as any, config:{sip_uri:"sip:1001@example.com", wss_url:"wss://pbx.example.com/wss", auth_user:"1001", password:"secret"}, isMuted:false,isHeld:false, connect:vi.fn(async()=>{}), disconnect:vi.fn(async()=>{}), call:callFn, hangup, answer, reject, mute:muteFn, hold:vi.fn(), sendDTMF:vi.fn(), duration:0}}>
        <WebPhonePopup open onOpenChange={()=>{}} />
      </WebPhoneContext.Provider>
    );
    render(<Wrapper/>);
    expect(await screen.findByTestId("incoming-banner")).toBeInTheDocument();
    expect(screen.getByText("Answer")).toBeInTheDocument();
    expect(screen.getByText("Reject")).toBeInTheDocument();
    fireEvent.click(screen.getByText("Answer"));
    expect(answer).toHaveBeenCalled();
    // dialpad
    expect(screen.getByTestId("webphone-dialpad")).toBeInTheDocument();
    expect(screen.getByTestId("dial-input")).toBeInTheDocument();
    expect(screen.getByTestId("hangup-btn")).toBeInTheDocument();
    // mute visible when not idle (incoming qualifies)
    expect(screen.getByText("Mute")).toBeInTheDocument();
    fireEvent.click(screen.getByText("Mute"));
    expect(muteFn).toHaveBeenCalledWith(true);
  });

  it("dialpad grid appends digits and call/hangup toggles", async()=>{
    const callFn = vi.fn(async()=>{}); const hangup = vi.fn(async()=>{});
    const IdleWrapper = ()=>(
      <WebPhoneContext.Provider value={{registrationState:"registered", registrationError:null, callState:"idle", currentSession:null, config:{sip_uri:"sip:1001@example.com", wss_url:"wss://pbx.example.com/wss", auth_user:"1001", password:"secret"}, isMuted:false,isHeld:false, connect:vi.fn(async()=>{}), disconnect:vi.fn(async()=>{}), call:callFn, hangup, answer:vi.fn(async()=>{}), reject:vi.fn(async()=>{}), mute:vi.fn(), hold:vi.fn(), sendDTMF:vi.fn(), duration:0}}>
        <WebPhonePopup open onOpenChange={()=>{}} />
      </WebPhoneContext.Provider>
    );
    const {unmount} = render(<IdleWrapper/>);
    const dialInput = await screen.findByTestId("dial-input") as HTMLInputElement;
    fireEvent.click(screen.getByText("1"));
    fireEvent.click(screen.getByText("2"));
    expect(dialInput.value).toBe("12");
    // call button disabled when empty? now has value, should be enabled
    const callBtn = screen.getByTestId("call-btn");
    expect(callBtn).not.toBeDisabled();
    fireEvent.click(callBtn);
    expect(callFn).toHaveBeenCalledWith("12");
    unmount();

    const ActiveWrapper = ()=>(
      <WebPhoneContext.Provider value={{registrationState:"registered", registrationError:null, callState:"confirmed", currentSession:{} as any, config:{sip_uri:"sip:1001@example.com", wss_url:"wss://pbx.example.com/wss", auth_user:"1001", password:"secret"}, isMuted:true,isHeld:false, connect:vi.fn(async()=>{}), disconnect:vi.fn(async()=>{}), call:callFn, hangup, answer:vi.fn(async()=>{}), reject:vi.fn(async()=>{}), mute:vi.fn(), hold:vi.fn(), sendDTMF:vi.fn(), duration:0}}>
        <WebPhonePopup open onOpenChange={()=>{}} />
      </WebPhoneContext.Provider>
    );
    render(<ActiveWrapper/>);
    expect(await screen.findByTestId("hangup-btn")).toBeInTheDocument();
    expect(screen.getByText("Unmute")).toBeInTheDocument();
  });
});
