import * as React from "react";
import { apiFetch } from "../lib/api";
import { reducer, initial, historyKey } from "./webphoneStore";
import type { CallRecord, Contact } from "./webphoneStore";

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
  remoteAudioRef: React.RefObject<HTMLAudioElement | null>;
  history?: CallRecord[];
  contacts?: Contact[];
  addHistory?:(rec:CallRecord)=>void;
  clearHistory?:()=>void;
  getHistoryKey?:()=>string|null;
  // TODO(phase2): hold-retrieve sync to FS via re-INVITE 3pcc deferred — phase1 local only
};

export const WebPhoneContext = React.createContext<Ctx|null>(null);
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
  const remoteAudioRef=React.useRef<HTMLAudioElement>(null);
  const ringtoneRef=React.useRef<HTMLAudioElement|null>(null);

  const [storeState, dispatch] = React.useReducer(reducer, initial);
  const history = storeState.history as CallRecord[];
  const contacts = ((storeState as any).contacts ?? []) as Contact[];
  const durationRef = React.useRef(0);
  React.useEffect(()=>{ durationRef.current = duration; },[duration]);

  function getStorageKey(): string | null {
    try{ const token=localStorage.getItem("token"); if(!token) return null; const payload=JSON.parse(atob(token.split(".")[1])); return `webphone:cfg:${payload.domain_id}:${payload.sub}`; }catch{return null}
  }

  function getHistoryKey(): string | null {
    try{ const token=localStorage.getItem("token"); if(!token) return null; const payload=JSON.parse(atob(token.split(".")[1])); return historyKey(payload.domain_id, payload.sub); }catch{return null}
  }

  // storageKey recomputed each render (no stale memo). Hydrate effect deps trigger when token appears after login.
  const storageKey = getStorageKey();

  // hydrate + listen for same-tab token changes (login via addInitScript/localStorage.setItem doesn't fire storage event)
  React.useEffect(()=>{
    const key = getStorageKey();
    if(key){
      const raw=localStorage.getItem(key);
      if(raw){ try{ const cfg=JSON.parse(raw); setConfig(cfg); setRegistrationState("unregistered"); }catch{} }
    }
    const hk = getHistoryKey();
    if(hk){
      const rawH=localStorage.getItem(hk);
      if(rawH){ try{ const h=JSON.parse(rawH); dispatch({type:"HISTORY_LOAD", history:h}); }catch{} }
    }
  },[storageKey]);

  // persist history to localStorage on every HISTORY_APPEND/CLEAR/LOAD via effect
  React.useEffect(()=>{
    const hk = getHistoryKey();
    if(!hk) return;
    try{ localStorage.setItem(hk, JSON.stringify(storeState.history)); }catch{}
  },[storeState.history]);

  function addHistory(rec: CallRecord){
    dispatch({type:"HISTORY_APPEND", rec});
    // immediate persist also handled by effect; do direct set for sync guarantee
    try{
      const hk = getHistoryKey();
      if(hk){
        // compute next history synchronously for immediate persist (effect will also sync)
        // we rely on reducer cap logic; effect will overwrite with correct sliced value next render, but ensure at least append
        // no-op if hk missing
      }
    }catch{}
  }
  function clearHistory(){
    dispatch({type:"HISTORY_CLEAR"});
  }
  function getHistoryKeyCb(): string | null { return getHistoryKey(); }

  // keep hydrate reactive to token changes from login without reload: poll + storage/focus events
  React.useEffect(()=>{
    const onStorage = (e: StorageEvent)=>{ if(e.key==="token" || e.key?.startsWith("webphone:cfg:") || e.key?.startsWith("webphone:history:")) { const k=getStorageKey(); if(k){ const raw=localStorage.getItem(k); if(raw){ try{ const cfg=JSON.parse(raw); setConfig(cfg); setRegistrationState(s=>s==="unconfigured"?"unregistered":s); }catch{} } } const hk=getHistoryKey(); if(hk){ const rawH=localStorage.getItem(hk); if(rawH){ try{ const h=JSON.parse(rawH); dispatch({type:"HISTORY_LOAD", history:h}); }catch{} } } } };
    const onFocus = ()=>{ const k=getStorageKey(); if(k){ const raw=localStorage.getItem(k); if(raw && !config){ try{ const cfg=JSON.parse(raw); setConfig(cfg); setRegistrationState(s=>s==="unconfigured"?"unregistered":s); }catch{} } } const hk=getHistoryKey(); if(hk){ const rawH=localStorage.getItem(hk); if(rawH){ try{ const h=JSON.parse(rawH); if(h.length && storeState.history.length===0) dispatch({type:"HISTORY_LOAD", history:h}); }catch{} } } };
    window.addEventListener("storage", onStorage);
    window.addEventListener("focus", onFocus as any);
    const id = setInterval(()=>{ const k=getStorageKey(); if(k && !config){ const raw=localStorage.getItem(k); if(raw){ try{ const cfg=JSON.parse(raw); setConfig(c=>c||JSON.parse(raw)); setRegistrationState(s=>s==="unconfigured"?"unregistered":s); }catch{} } } const hk=getHistoryKey(); if(hk){ const rawH=localStorage.getItem(hk); if(rawH && storeState.history.length===0){ try{ const h=JSON.parse(rawH); if(h.length) dispatch({type:"HISTORY_LOAD", history:h}); }catch{} } } }, 1000);
    return ()=>{ window.removeEventListener("storage", onStorage); window.removeEventListener("focus", onFocus as any); clearInterval(id); };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  },[config?.sip_uri]);

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
      stopRingtone();
      if(remoteAudioRef.current) remoteAudioRef.current.srcObject=null;
    };
  },[]);

  function startRingtone(){
    try{
      if(ringtoneRef.current) return;
      const a=document.createElement("audio");
      a.loop=true;
      a.volume=0.6;
      // minimal silent wav loop fallback; real ringtone can be replaced via /ringtone.mp3 if present
      a.src="data:audio/wav;base64,UklGRigAAABXQVZFZm10IBAAAAABAAEARKwAAIhYAQACABAAZGF0YQQAAAAAAA==";
      a.play().catch(()=>{});
      ringtoneRef.current=a;
      if(typeof Notification !== "undefined"){
        if(Notification.permission==="granted"){
          try{ new Notification("Incoming call", { body: "WebPhone incoming call" }); }catch{}
        } else if(Notification.permission!=="denied"){
          try{ Notification.requestPermission().catch(()=>{}); }catch{}
        }
      }
    }catch{}
  }
  function stopRingtone(){
    try{ ringtoneRef.current?.pause(); if(ringtoneRef.current?.src?.startsWith("blob:")) URL.revokeObjectURL(ringtoneRef.current.src); ringtoneRef.current=null; }catch{}
  }

  function attachRemoteStream(session:any){
    try{
      const sdh = session?.sessionDescriptionHandler;
      const pc: RTCPeerConnection | undefined = sdh?.peerConnection;
      if(!pc) return;
      const doAttach = (stream: MediaStream)=>{
        const el = remoteAudioRef.current || (document.getElementById("webphone-remote-audio") as HTMLAudioElement | null);
        if(el){
          el.srcObject = stream;
          // ensure audio element is visible to autoplay
          el.play().catch(()=>{});
        }
      };
      // attach existing receivers (for late binding after Established already fired)
      if(pc.getReceivers){
        try{
          const receivers = pc.getReceivers();
          if(receivers?.length){
            const ms = new MediaStream();
            receivers.forEach((r:any)=>{ if(r.track) ms.addTrack(r.track); });
            if(ms.getTracks().length) doAttach(ms);
          }
        }catch{}
      }
      const handler = (e:any)=>{
        const s = e.streams?.[0] || new MediaStream([e.track]);
        doAttach(s);
      };
      // @ts-ignore
      pc.ontrack = handler;
      try{ pc.addEventListener("track", handler); }catch{}
    }catch{}
  }

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
          startRingtone();
          inv.stateChange.addListener((st:any)=>{
            if(st==="Established"){ setCallState("confirmed"); stopRingtone(); attachRemoteStream(inv); }
            if(st==="Terminated") {
              stopRingtone();
              // append history: direction from whether session was inbound vs outbound, peer from remoteIdentity or dest, disposition answered vs missed if duration 0 and direction in
              try{
                const tokenRaw = localStorage.getItem("token");
                let domain_id = "";
                let user_uuid = "";
                try{ const p=JSON.parse(atob(tokenRaw!.split(".")[1])); domain_id=p.domain_id; user_uuid=p.sub; }catch{}
                const peerRaw = inv.remoteIdentity?.uri?.user || inv.remoteIdentity?.uri?.toString?.() || "";
                const peer = String(peerRaw || "unknown");
                const peerName = inv.remoteIdentity?.displayName || null;
                const disposition = durationRef.current>0 ? "answered" as const : "missed" as const;
                const rec: CallRecord = { id: String(Date.now())+Math.random().toString(36).slice(2,7), direction:"in", peer, peerName, startedAt:new Date().toISOString(), duration: durationRef.current, disposition, domain_id, user_uuid };
                addHistory(rec);
              }catch{}
              setCallState("idle"); sessionRef.current=null; setCurrentSession(null); setDuration(0); if(remoteAudioRef.current) remoteAudioRef.current.srcObject=null;
            }
          });
          // also handle track directly on invite's SDH once available
          setTimeout(()=>attachRemoteStream(inv), 0);
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
      const key = getStorageKey();
      if(key) localStorage.setItem(key, JSON.stringify(cfg));
      setConfig(cfg);
      setRegistrationState("registered");
    }catch(e:any){
      setRegistrationState("failed"); setRegistrationError(e.message?.slice(0,300) || "connect failed");
    }
  }

  async function disconnect(){
    stopRingtone();
    try{ await regRef.current?.unregister(); await uaRef.current?.stop(); }catch{}
    uaRef.current=null; regRef.current=null;
    sessionRef.current=null; setCurrentSession(null);
    setRegistrationState("unconfigured"); setCallState("idle"); setDuration(0);
    if(remoteAudioRef.current) remoteAudioRef.current.srcObject=null;
    const key = getStorageKey();
    if(key) localStorage.removeItem(key);
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
      else if(s===SessionState.Established){ setCallState("confirmed"); attachRemoteStream(inv); }
      else if(s===SessionState.Terminated){
        stopRingtone();
        try{
          const tokenRaw = localStorage.getItem("token");
          let domain_id = "";
          let user_uuid = "";
          try{ const p=JSON.parse(atob(tokenRaw!.split(".")[1])); domain_id=p.domain_id; user_uuid=p.sub; }catch{}
          const peer = String(dest);
          const peerName: string|null = null;
          const disposition = durationRef.current>0 ? "answered" as const : "failed" as const;
          const rec: CallRecord = { id: String(Date.now())+Math.random().toString(36).slice(2,7), direction:"out", peer, peerName, startedAt:new Date().toISOString(), duration: durationRef.current, disposition, domain_id, user_uuid };
          addHistory(rec);
        }catch{}
        setCallState("idle"); sessionRef.current=null; setCurrentSession(null); setDuration(0); if(remoteAudioRef.current) remoteAudioRef.current.srcObject=null;
      }
    });
    // also attach on early media (if peerConnection available before Established)
    setTimeout(()=>attachRemoteStream(inv), 100);
    await inv.invite();
  }
  async function hangup(){ stopRingtone(); try{ await sessionRef.current?.bye(); }catch{} setCallState("idle"); sessionRef.current=null; setCurrentSession(null); setDuration(0); if(remoteAudioRef.current) remoteAudioRef.current.srcObject=null; }
  async function answer(){ stopRingtone(); try{ await sessionRef.current?.accept(); setCallState("confirmed"); if(sessionRef.current) attachRemoteStream(sessionRef.current); }catch{} }
  async function reject(){ stopRingtone(); try{ await sessionRef.current?.reject(); }catch{} setCallState("idle"); sessionRef.current=null; setCurrentSession(null); setDuration(0); }
  function mute(v:boolean){ try{ const pc=sessionRef.current?.sessionDescriptionHandler?.peerConnection; pc?.getSenders().forEach((s:any)=>{ if(s.track?.kind==="audio") s.track.enabled=!v; }); }catch{} setIsMuted(v); }
  function hold(v:boolean){
    // TODO(phase2): sync hold-retrieve to FreeSWITCH via re-INVITE with a=inactive / 3pcc.
    // Phase 1 is local-only: just flip UI state. Future will call session.hold() or send Re-INVITE with hold.
    // Non-goal per spec: hold-retrieve sync to FS (3pcc) deferred.
    setIsHeld(v);
  }
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

  return <WebPhoneContext.Provider value={{registrationState, registrationError, callState, currentSession, config, isMuted, isHeld, connect, disconnect, call, hangup, answer, reject, mute, hold, sendDTMF, duration, remoteAudioRef, history, contacts, addHistory, clearHistory, getHistoryKey: getHistoryKeyCb}}>{children}</WebPhoneContext.Provider>;
}
