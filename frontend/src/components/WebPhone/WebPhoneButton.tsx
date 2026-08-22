import * as React from "react";
import { Phone } from "lucide-react";
import { useWebPhone } from "../../context/WebPhoneContext";

export function WebPhoneButton({onOpen}:{onOpen:()=>void}){
  const {registrationState, registrationError} = useWebPhone();
  let dot="bg-gray-400"; let aria="webphone unconfigured"; let label="";
  if(registrationState==="connecting"){ dot="bg-yellow-500 animate-pulse"; aria="webphone connecting"; }
  else if(registrationState==="registered"){ dot="bg-green-500 shadow-[0_0_6px_rgba(34,197,94,0.6)]"; aria="webphone registered"; }
  else if(registrationState==="failed"||registrationState==="unregistered"){ dot="bg-red-500"; aria="webphone disconnected"; }
  const tooltip = registrationError ? `WebPhone ${registrationState}: ${registrationError}` : `WebPhone ${registrationState}`;
  return (
    <button data-testid="webphone-btn" onClick={onOpen} title={tooltip} aria-label={aria} className="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-medium hover:bg-muted">
      <span data-testid="webphone-registration-dot" aria-hidden className={`inline-block h-2 w-2 rounded-full ${dot}`} />
      <Phone className="h-3.5 w-3.5" />
      <span className="hidden sm:inline">{registrationState==="registered"? "Registered": registrationState==="connecting"? "Connecting": "Phone"}</span>
    </button>
  );
}
