import * as React from "react";
import { WebPhonePanel } from "./WebPhonePanel";

export function WebPhonePopup(props:{open:boolean; onOpenChange:(v:boolean)=>void; embedded?:boolean}){
  return <WebPhonePanel {...props} />;
}
