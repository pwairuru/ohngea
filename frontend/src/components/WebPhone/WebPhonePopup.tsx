import * as React from "react";
import { Dialog, DialogHeader, DialogTitle } from "../ui/dialog";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { useWebPhone } from "../../context/WebPhoneContext";
import { apiFetch } from "../../lib/api";

export function WebPhonePopup({open, onOpenChange}:{open:boolean; onOpenChange:(v:boolean)=>void}){
  const {registrationState, registrationError, config, connect, disconnect, call, hangup, answer, reject, callState, isMuted, mute, duration, remoteAudioRef, hold, isHeld} = useWebPhone();
  const [settings,setSettings]=React.useState<any>(null);
  const [extList,setExtList]=React.useState<any[]>([]);
  const [form,setForm]=React.useState({extension:"", webrtc_user:"", webrtc_pass:"", wss_url:"", useExtension:true});
  const [dial,setDial]=React.useState("");
  const [loading,setLoading]=React.useState(false);
  const [error,setError]=React.useState<string|null>(null);

  React.useEffect(()=>{
    if(!open) return;
    const domainId = localStorage.getItem("domain_id") || (()=>{
      try{ return JSON.parse(atob(localStorage.getItem("token")!.split(".")[1])).domain_id }catch{return null}
    })();
    if(domainId) apiFetch(`/domains/${domainId}/webrtc_settings`).then(setSettings).catch(()=>setSettings({enabled:false}));
    apiFetch("/extensions?per_page=100").then((r:any)=>setExtList(r.data||[])).catch(()=>{});
  },[open]);

  const isConfigured = !!config && registrationState!=="unconfigured";

  async function handleSave(){
    // validate wss_url when in custom SIP mode
    if (!form.useExtension) {
      const urlToValidate = form.wss_url || settings?.wss_url || "";
      if (urlToValidate && !urlToValidate.startsWith("wss://")) {
        setError("WSS URL must start with wss://");
        return;
      }
      if (!form.webrtc_user || !form.webrtc_pass) {
        // allow missing to be caught by server, but still show generic validation if both empty and no wss error?
        // don't block - server will error
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

  const timer = `${String(Math.floor(duration/60)).padStart(2,"0")}:${String(duration%60).padStart(2,"0")}`;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogHeader><DialogTitle>WebPhone {isConfigured? `— ${config?.auth_user}`: ""}</DialogTitle></DialogHeader>
      {callState==="incoming" && (
        <div data-testid="incoming-banner" className="rounded border bg-yellow-50 p-3 flex justify-between">
          <span>Incoming call</span>
          <span className="flex gap-2"><Button onClick={answer} className="bg-green-600">Answer</Button><Button onClick={reject} variant="destructive">Reject</Button></span>
        </div>
      )}
      {!isConfigured ? (
        <div data-testid="webphone-config-form" className="flex flex-col gap-3">
          {settings?.auth_mode==="both" && (
            <div className="flex gap-2"><Button variant={form.useExtension?"default":"outline"} onClick={()=>setForm(f=>({...f,useExtension:true}))}>Extension</Button><Button variant={!form.useExtension?"default":"outline"} onClick={()=>setForm(f=>({...f,useExtension:false}))}>Custom SIP</Button></div>
          )}
          {(settings?.auth_mode==="extension" || (settings?.auth_mode==="both" && form.useExtension)) ? (
            <>
              <Label>Extension</Label>
              <select data-testid="ext-select" value={form.extension} onChange={e=>setForm(f=>({...f,extension:e.target.value}))} className="border rounded px-3 py-2">
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
        <div data-testid="webphone-dialpad" className="flex flex-col gap-3">
          <div className="flex items-center justify-between text-xs"><span className="flex items-center gap-1"><span className="h-2 w-2 rounded-full bg-green-500 inline-block"/>Registered as {config?.auth_user}</span><button onClick={()=>disconnect()} className="underline">Unregister</button></div>
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
      )}
    </Dialog>
  );
}
