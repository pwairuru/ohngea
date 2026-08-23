import * as React from "react";
import { PhoneOff, Mic, MicOff, Maximize2 } from "lucide-react";
import { useWebPhone } from "../../context/WebPhoneContext";
export function WebPhoneDock({onExpand}:{onExpand:()=>void}){
  const {registrationState, callState, duration, config, isMuted, mute, hangup}=useWebPhone();
  const [min,setMin]=React.useState(()=> typeof window==="undefined" ? true : localStorage.getItem("webphone:dock:state")!=="hidden");
  React.useEffect(()=>{ if(typeof window!=="undefined") localStorage.setItem("webphone:dock:state", min?"shown":"hidden"); },[min]);
  if(typeof window!=="undefined" && window.location.pathname.startsWith("/login")) return null;
  let dot="bg-gray-400"; if(registrationState==="connecting") dot="bg-yellow-500 animate-pulse"; else if(registrationState==="registered") dot="bg-green-500"; else if(registrationState==="failed"||registrationState==="unregistered") dot="bg-red-500";
  const timer=`${String(Math.floor(duration/60)).padStart(2,"0")}:${String(duration%60).padStart(2,"0")}`;
  if(!min) return <button data-testid="webphone-dock-mini" onClick={()=>setMin(true)} className="fixed bottom-4 right-4 h-10 w-10 rounded-full bg-primary text-primary-foreground">☎</button>;
  return <div data-testid="webphone-dock" className="fixed bottom-4 right-4 z-30 flex items-center gap-2 rounded-full border bg-background px-3 py-2 shadow-lg">
    <span data-testid="webphone-dock-dot" className={`h-2 w-2 rounded-full ${dot}`} />
    <span className="text-xs hidden sm:inline">{config?.auth_user||"Phone"} {callState==="confirmed"? timer : registrationState}</span>
    {callState==="confirmed"&& <span className="font-mono text-xs">{timer}</span>}
    {callState!=="idle"&& <button data-testid="dock-mute" onClick={()=>mute(!isMuted)} className="h-8 w-8 border rounded flex items-center justify-center">{isMuted?<MicOff className="h-4 w-4"/>:<Mic className="h-4 w-4"/>}</button>}
    {callState!=="idle"&& <button data-testid="dock-hangup" onClick={hangup} className="h-8 w-8 bg-red-600 text-white rounded flex items-center justify-center"><PhoneOff className="h-4 w-4"/></button>}
    <button data-testid="dock-expand" onClick={onExpand} className="h-8 w-8 border rounded flex items-center justify-center"><Maximize2 className="h-4 w-4"/></button>
    <button onClick={()=>setMin(false)} className="text-xs underline">—</button>
  </div>;
}
