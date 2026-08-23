import * as React from "react";
import { Dialog, DialogHeader, DialogTitle } from "../ui/dialog";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "../ui/tabs";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { useWebPhone } from "../../context/WebPhoneContext";
import { apiFetch } from "../../lib/api";
import { HistoryView } from "./HistoryView";
import { ContactsView } from "./ContactsView";
import { DevicePicker } from "./DevicePicker";

export function WebPhonePanel({open, onOpenChange, embedded}:{open:boolean; onOpenChange:(v:boolean)=>void; embedded?:boolean}){
  const {registrationState, registrationError, config, connect, disconnect, call, hangup, answer, reject, callState, isMuted, mute, duration, remoteAudioRef, hold, isHeld, history, clearHistory} = useWebPhone();
  const [settings,setSettings]=React.useState<any>(null);
  const [extList,setExtList]=React.useState<any[]>([]);
  const [form,setForm]=React.useState({extension:"", webrtc_user:"", webrtc_pass:"", wss_url:"", useExtension:true});
  const [dial,setDial]=React.useState("");
  const [loading,setLoading]=React.useState(false);
  const [error,setError]=React.useState<string|null>(null);
  const [contacts,setContacts]=React.useState<any[]>([]);

  const isConfigured = !!config && registrationState!=="unconfigured";
  const timer = `${String(Math.floor(duration/60)).padStart(2,"0")}:${String(duration%60).padStart(2,"0")}`;

  const [activeTab,setActiveTab]=React.useState<string>(()=> isConfigured ? "dialpad" : "settings");
  const prevOpen=React.useRef(open);
  React.useEffect(()=>{ if(open && !prevOpen.current) setActiveTab(isConfigured?"dialpad":"settings"); prevOpen.current=open; },[open]);

  React.useEffect(()=>{
    if(!open) return;
    let cancelled=false;
    const domainId = localStorage.getItem("domain_id") || (()=>{
      try{ return JSON.parse(atob(localStorage.getItem("token")!.split(".")[1])).domain_id }catch{return null}
    })();
    if(domainId) apiFetch(`/domains/${domainId}/webrtc_settings`).then(s=>{ if(!cancelled) setSettings(s); }).catch(()=>{ if(!cancelled) setSettings({enabled:false}); });
    async function load(){
      try{
        const token=localStorage.getItem("token");
        let domain="", user="";
        try{ const p=JSON.parse(atob(token!.split(".")[1])); domain=p.domain_id; user=p.sub; }catch{}
        const favKey=`webphone:favorites:${domain}:${user}`;
        const [extRes, userRes]=await Promise.all([
          apiFetch("/extensions?per_page=100").catch(()=>({data:[]})),
          apiFetch("/users?per_page=100").catch(()=> apiFetch("/users").catch(()=>({data:[]}))),
        ]);
        if(cancelled) return;
        const extData=(extRes as any)?.data || (Array.isArray(extRes)?extRes:[]);
        const userData=(userRes as any)?.data || (Array.isArray(userRes)?userRes:[]);
        if(!cancelled) setExtList(extData);
        let favIds:string[]=[];
        try{ const raw=localStorage.getItem(favKey); if(raw) favIds=JSON.parse(raw); }catch{}
        const favSet=new Set(favIds);
        const mappedExt=extData.map((e:any)=>({ id:e.extension_uuid || e.id || e.extension, kind:"extension", primary:e.extension, name:e.effective_caller_id_name || e.name || e.extension, favorite: favSet.has(e.extension_uuid || e.id || e.extension), domain_id: e.domain_id || domain }));
        const mappedUsers=userData.map((u:any)=>({ id:u.user_uuid || u.id, kind:"user", primary:u.username || u.extension || u.primary, name:u.contact_name || u.username || u.primary, favorite: favSet.has(u.user_uuid || u.id), domain_id: u.domain_id || domain }));
        const merged=[...mappedExt, ...mappedUsers].filter((c:any)=>c.primary).map((c:any)=>({...c, favorite: favSet.has(c.id)}));
        if(cancelled) return;
        setContacts(prev=>{
          if(cancelled) return prev;
          let latestIds:string[]=[];
          try{ const raw=localStorage.getItem(favKey); if(raw) latestIds=JSON.parse(raw); }catch{}
          const latestSet=new Set(latestIds);
          const withLatest=merged.map((c:any)=>({...c, favorite: latestSet.has(c.id)}));
          if(prev.length===0) return withLatest;
          return prev.map((p:any)=>({...p, favorite: latestSet.has(p.id)}));
        });
      }catch{
        if(!cancelled) setContacts([]);
      }
    }
    load();
    return ()=>{ cancelled=true; };
  },[open]);

  React.useEffect(()=>{
    const h=(e:KeyboardEvent)=>{
      if((e.ctrlKey||e.metaKey)&&e.key==='k'){
        const dial=document.querySelector<HTMLInputElement>('[data-testid="dial-input"]');
        if(dial){ e.preventDefault(); (e as any).stopImmediatePropagation?.(); dial.focus(); }
      }
      if(e.key==='Escape'&&callState!=="idle") hangup();
    };
    window.addEventListener("keydown",h);
    return()=>window.removeEventListener("keydown",h);
  },[callState]);

  function handleToggleFav(id:string){
    try{
      const token=localStorage.getItem("token");
      let domain="", user="";
      try{ const p=JSON.parse(atob(token!.split(".")[1])); domain=p.domain_id; user=p.sub; }catch{}
      const favKey=`webphone:favorites:${domain}:${user}`;
      let favIds:string[]=[];
      try{ const raw=localStorage.getItem(favKey); if(raw) favIds=JSON.parse(raw); }catch{}
      const set=new Set(favIds);
      const isFav=set.has(id);
      if(isFav) set.delete(id); else set.add(id);
      const nextIds=Array.from(set);
      localStorage.setItem(favKey, JSON.stringify(nextIds));
      setContacts(prev=> prev.map(c=> c.id===id ? {...c, favorite: !isFav } : c));
    }catch{}
  }

  function handleRedial(peer:string){
    setDial(peer);
    setActiveTab("dialpad");
  }
  function handleContactDial(primary:string){
    setDial(primary);
    setActiveTab("dialpad");
  }

  async function handleSave(){
    if (!form.useExtension) {
      const urlToValidate = form.wss_url || settings?.wss_url || "";
      if (urlToValidate && !urlToValidate.startsWith("wss://") && !urlToValidate.startsWith("ws://")) {
        setError("WSS URL must start with wss:// or ws://");
        return;
      }
      if (urlToValidate.startsWith("ws://") && !urlToValidate.includes("localhost") && !urlToValidate.includes("127.0.0.1")) {
        setError("ws:// only allowed for localhost — use wss:// for production");
        return;
      }
    } else {
      if (!form.extension) {
        setError("Please select an extension");
        return;
      }
    }
    setLoading(true); setError(null);
    try{
      const body = form.useExtension ? {extension: form.extension} : {webrtc_user: form.webrtc_user, webrtc_pass: form.webrtc_pass};
      const creds = await apiFetch("/webrtc/credentials", {method:"POST", body: JSON.stringify(body)});
      await connect(creds);
    }catch(e:any){ setError(e.message?.slice(0,300)||"failed"); }
    finally{ setLoading(false); }
  }

  const content = (
    <>
      <DialogHeader><DialogTitle>WebPhone {isConfigured? `— ${config?.auth_user}`: ""}</DialogTitle></DialogHeader>
      {callState==="incoming" && (
        <div data-testid="incoming-banner" className="rounded border bg-yellow-50 p-3 flex justify-between">
          <span>Incoming call</span>
          <span className="flex gap-2"><Button onClick={answer} className="bg-green-600">Answer</Button><Button onClick={reject} variant="destructive">Reject</Button></span>
        </div>
      )}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
        <TabsList>
          <TabsTrigger triggerValue="dialpad">Dialpad</TabsTrigger>
          <TabsTrigger triggerValue="history">History</TabsTrigger>
          <TabsTrigger triggerValue="contacts">Contacts</TabsTrigger>
          <TabsTrigger triggerValue="settings">Settings</TabsTrigger>
        </TabsList>
        <TabsContent contentValue="dialpad">
          <div data-testid="webphone-dialpad" className="flex flex-col gap-3">
            <div className="flex items-center justify-between text-xs"><span className="flex items-center gap-1"><span className="h-2 w-2 rounded-full bg-green-500 inline-block"/>Registered as {config?.auth_user}</span><button onClick={()=>disconnect()} className="underline min-h-[44px]">Unregister</button></div>
            <div className="flex items-center justify-between">
              <Input data-testid="dial-input" value={dial} onChange={e=>setDial(e.target.value)} placeholder="1001" className="font-mono" />
              <Button variant="ghost" onClick={()=>setDial("")}>Clear</Button>
            </div>
            <div className="grid grid-cols-3 gap-2">
              {["1","2","3","4","5","6","7","8","9","*","0","#"].map(k=>(
                <Button key={k} variant="outline" className="h-12 text-lg" onClick={()=>setDial(d=>d+k)}>{k}</Button>
              ))}
            </div>
            <div className="flex gap-2">
              {callState==="idle" ? <Button data-testid="call-btn" onClick={()=>call(dial)} disabled={!dial} className="flex-1 bg-green-600 hover:bg-green-700">Call</Button> : <Button data-testid="hangup-btn" onClick={hangup} variant="destructive" className="flex-1">Hangup</Button>}
              {callState!=="idle" && <Button onClick={()=>mute(!isMuted)} variant="outline">{isMuted?"Unmute":"Mute"}</Button>}
              {callState!=="idle" && <Button onClick={()=>hold(!isHeld)} variant="outline" title="Local hold only — FS sync deferred to phase2 (TODO)">{isHeld?"Unhold":"Hold"}</Button>}
            </div>
            {callState==="confirmed" && <div data-testid="call-timer" className="text-sm font-mono text-center">{timer}</div>}
            <div className="text-xs text-muted-foreground">State: {registrationState} • Call: {callState} {callState==="confirmed" ? `• ${timer}` : ""} {registrationError? `• ${registrationError}`:""}</div>
            <audio ref={remoteAudioRef} id="webphone-remote-audio" autoPlay hidden />
          </div>
        </TabsContent>
        <TabsContent contentValue="history">
          <HistoryView history={history} contacts={contacts} onRedial={handleRedial} onClear={clearHistory} />
        </TabsContent>
        <TabsContent contentValue="contacts">
          <ContactsView contacts={contacts} onDial={handleContactDial} onToggleFav={handleToggleFav} />
        </TabsContent>
        <TabsContent contentValue="settings">
          {!isConfigured ? (
            <div data-testid="webphone-config-form" className="flex flex-col gap-3">
              {settings?.auth_mode==="both" && (
                <div className="flex gap-2"><Button variant={form.useExtension?"default":"outline"} onClick={()=>setForm(f=>({...f,useExtension:true}))}>Extension</Button><Button variant={!form.useExtension?"default":"outline"} onClick={()=>setForm(f=>({...f,useExtension:false}))}>Custom SIP</Button></div>
              )}
              {(settings?.auth_mode==="extension" || (settings?.auth_mode==="both" && form.useExtension)) ? (
                <>
                  <Label>Extension</Label>
                  <select data-testid="ext-select" value={form.extension} onChange={e=>setForm(f=>({...f,extension:e.target.value}))} className="border rounded px-3 py-2 min-h-[44px]">
                    <option value="">Select extension</option>
                    {extList.map((e:any)=><option key={e.extension_uuid} value={e.extension}>{e.extension} — {e.effective_caller_id_name}</option>)}
                  </select>
                  <div className="text-xs text-muted-foreground">WSS: {settings?.wss_url || "not configured"}</div>
                </>
              ) : (
                <>
                  <Label>SIP User</Label><Input value={form.webrtc_user} onChange={e=>setForm(f=>({...f,webrtc_user:e.target.value}))} placeholder="1001" />
                  <Label>Password</Label><Input type="password" value={form.webrtc_pass} onChange={e=>setForm(f=>({...f,webrtc_pass:e.target.value}))} />
                  <Label>WSS URL</Label><Input value={form.wss_url || settings?.wss_url || ""} onChange={e=>setForm(f=>({...f,wss_url:e.target.value}))} placeholder="wss://pbx.example.com/wss" />
                </>
              )}
              {error && <div className="text-sm text-red-600">{error}</div>}
              {registrationError && <div className="text-sm text-red-600">{registrationError}</div>}
              <Button data-testid="save-register" onClick={handleSave} disabled={loading}>{loading?"Registering...":"Save & Register"}</Button>
              <div className="text-xs text-muted-foreground">Auth mode: {settings?.auth_mode || "…"}</div>
            </div>
          ) : (
            <div className="flex flex-col gap-3">
              <div className="text-sm">Registered as {config?.auth_user}</div>
              <Button variant="outline" onClick={()=>disconnect()}>Unregister</Button>
              <DevicePicker />
              <div className="text-xs text-muted-foreground">WSS: {settings?.wss_url || config?.wss_url || "not configured"}</div>
              <div className="text-xs text-muted-foreground">Auth mode: {settings?.auth_mode || "…"}</div>
              {registrationError && <div className="text-sm text-red-600">{registrationError}</div>}
            </div>
          )}
        </TabsContent>
      </Tabs>
    </>
  );

  if(embedded){
    return <div data-testid="webphone-panel" className="border rounded-lg p-4 bg-background">{content}</div>;
  }
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <div data-testid="webphone-panel">
        {content}
      </div>
    </Dialog>
  );
}
