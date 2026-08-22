import * as React from "react";
import { apiFetch } from "../lib/api";

export type RegistrationState = "unconfigured"|"connecting"|"registered"|"unregistered"|"failed";
export type CallState = "idle"|"trying"|"proceeding"|"early"|"confirmed"|"terminated"|"incoming";
export type WebPhoneConfig = { sip_uri:string; wss_url:string; auth_user:string; password:string; display_name?:string; ice_servers?: any[] };

type Ctx = {
  registrationState: RegistrationState;
  registrationError: string|null;
  callState: CallState;
  currentSession: any | null;
  config: WebPhoneConfig|null;
  isMuted:boolean; isHeld:boolean;
  connect:(cfg:WebPhoneConfig)=>Promise<void>;
  disconnect:()=>Promise<void>;
  call:(dest:string)=>Promise<void>;
  hangup:()=>Promise<void>;
  answer:()=>Promise<void>;
  reject:()=>Promise<void>;
  mute:(v:boolean)=>void;
  hold:(v:boolean)=>void;
  sendDTMF:(tone:string)=>void;
  duration:number;
};

const WebPhoneContext = React.createContext<Ctx|null>(null);
export function useWebPhone(){ const c=React.useContext(WebPhoneContext); if(!c) throw new Error("missing provider"); return c; }

export function WebPhoneProvider({children}:{children:React.ReactNode}){
  const [registrationState,setRegistrationState]=React.useState<RegistrationState>("unconfigured");
  const [registrationError,setRegistrationError]=React.useState<string|null>(null);
  const [callState,setCallState]=React.useState<CallState>("idle");
  const [config,setConfig]=React.useState<WebPhoneConfig|null>(null);
  const [isMuted,setIsMuted]=React.useState(false);
  const [isHeld,setIsHeld]=React.useState(false);
  const [duration,setDuration]=React.useState(0);
  const [currentSession,setCurrentSession]=React.useState<any>(null);
  const uaRef=React.useRef<any>(null);
  const regRef=React.useRef<any>(null);
  const sessionRef=React.useRef<any>(null);

  const storageKey=React.useMemo(()=>{
    try{ const token=localStorage.getItem("token"); if(!token) return null; const payload=JSON.parse(atob(token.split(".")[1])); return `webphone:cfg:${payload.domain_id}:${payload.sub}`; }catch{return null}
  },[]);

  // hydrate
  React.useEffect(()=>{
    if(!storageKey) return;
    const raw=localStorage.getItem(storageKey);
    if(raw){ try{ const cfg=JSON.parse(raw); setConfig(cfg); setRegistrationState("unregistered"); }catch{} }
  },[storageKey]);

  // auto-connect if config + enabled
  React.useEffect(()=>{
    if(!config) return;
    const domainId = localStorage.getItem("domain_id") || (()=>{
      try{ return JSON.parse(atob(localStorage.getItem("token")!.split(".")[1])).domain_id }catch{return null}
    })();
    if(!domainId) return;
    let cancelled = false;
    apiFetch(`/domains/${domainId}/webrtc_settings`).then((s:any)=>{
      if(cancelled) return;
      if(s?.enabled) connect(config);
    }).catch((e:any)=>{
      if(!cancelled) console.warn("webrtc_settings fetch failed", e?.message || e);
    });
    return ()=>{ cancelled = true; };
    // eslint-disable-next-line
  },[config?.sip_uri]);

  // provider unmount cleanup
  React.useEffect(()=>{
    return ()=>{
      try{ regRef.current?.unregister(); }catch{}
      try{ uaRef.current?.stop(); }catch{}
      uaRef.current=null; regRef.current=null;
    };
  },[]);

  async function connect(cfg:WebPhoneConfig){
    setRegistrationState("connecting"); setRegistrationError(null);
    try{
      const { UserAgent, Registerer } = await import("sip.js");
      const uri = UserAgent.makeURI(cfg.sip_uri);
      if(!uri) throw new Error("Invalid SIP URI");
      const ua = new UserAgent({
        uri,
        transportOptions: { server: cfg.wss_url, iceServers: cfg.ice_servers },
        authorizationUsername: cfg.auth_user,
        authorizationPassword: cfg.password,
        displayName: cfg.display_name,
      } as any);
      ua.delegate = {
        onInvite: (inv:any)=>{
          sessionRef.current=inv;
          setCurrentSession(inv);
          setCallState("incoming");
          inv.stateChange.addListener((st:any)=>{ if(st==="Established") setCallState("confirmed"); if(st==="Terminated") {setCallState("idle"); sessionRef.current=null; setCurrentSession(null); setDuration(0);} });
        }
      };
      await ua.start();
      const registerer = new Registerer(ua);
      // @ts-ignore
      registerer.stateChange.addListener((state:string)=>{
        if(state==="Registered") setRegistrationState("registered");
        else if(state==="Unregistered") setRegistrationState("unregistered");
        else if(state==="Terminated") setRegistrationState("failed");
      });
      await registerer.register();
      uaRef.current=ua; regRef.current=registerer;
      if(storageKey) localStorage.setItem(storageKey, JSON.stringify(cfg));
      setConfig(cfg);
      setRegistrationState("registered");
    }catch(e:any){
      setRegistrationState("failed"); setRegistrationError(e.message?.slice(0,300) || "connect failed");
    }
  }

  async function disconnect(){
    try{ await regRef.current?.unregister(); await uaRef.current?.stop(); }catch{}
    uaRef.current=null; regRef.current=null;
    sessionRef.current=null; setCurrentSession(null);
    setRegistrationState("unconfigured"); setCallState("idle"); setDuration(0);
    if(storageKey) localStorage.removeItem(storageKey);
    setConfig(null);
  }
  async function call(dest:string){
    if(!uaRef.current) throw new Error("not registered");
    const { Inviter, SessionState, UserAgent } = await import("sip.js");
    const uri = UserAgent.makeURI(`sip:${dest}@${config?.sip_uri.split("@")[1]}`);
    if(!uri) throw new Error("bad dest");
    const inv = new Inviter(uaRef.current, uri, { sessionDescriptionHandlerOptions: { constraints: {audio:true, video:false}}});
    sessionRef.current=inv;
    setCurrentSession(inv);
    setCallState("trying");
    setDuration(0);
    inv.stateChange.addListener((s:string)=>{
      if(s===SessionState.Establishing) setCallState("proceeding");
      else if(s===SessionState.Established) setCallState("confirmed");
      else if(s===SessionState.Terminated){ setCallState("idle"); sessionRef.current=null; setCurrentSession(null); setDuration(0); }
    });
    await inv.invite();
  }
  async function hangup(){ try{ await sessionRef.current?.bye(); }catch{} setCallState("idle"); sessionRef.current=null; setCurrentSession(null); setDuration(0); }
  async function answer(){ try{ await sessionRef.current?.accept(); setCallState("confirmed"); }catch{} }
  async function reject(){ try{ await sessionRef.current?.reject(); }catch{} setCallState("idle"); sessionRef.current=null; setCurrentSession(null); setDuration(0); }
  function mute(v:boolean){ try{ const pc=sessionRef.current?.sessionDescriptionHandler?.peerConnection; pc?.getSenders().forEach((s:any)=>{ if(s.track?.kind==="audio") s.track.enabled=!v; }); }catch{} setIsMuted(v); }
  function hold(v:boolean){ setIsHeld(v); /* sip.js hold via invite with a=inactive - phase2 */ }
  function sendDTMF(tone:string){ try{ sessionRef.current?.sessionDescriptionHandler?.sendDtmf(tone); }catch{} }

  // duration ticker
  React.useEffect(()=>{
    if(callState!=="confirmed") {
      if(callState==="idle" || callState==="terminated") setDuration(0);
      return;
    }
    const id=setInterval(()=>setDuration(d=>d+1),1000);
    return()=>clearInterval(id);
  },[callState]);

  return <WebPhoneContext.Provider value={{registrationState, registrationError, callState, currentSession, config, isMuted, isHeld, connect, disconnect, call, hangup, answer, reject, mute, hold, sendDTMF, duration}}>{children}</WebPhoneContext.Provider>;
}
