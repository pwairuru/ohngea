import { WebPhonePanel } from "../components/WebPhone/WebPhonePanel";
import * as React from "react";
export function Phone(){
  const [open,setOpen]=React.useState(true);
  return <div className="p-4 grid md:grid-cols-2 gap-4" data-testid="phone-page"><div><WebPhonePanel open={open} onOpenChange={setOpen} embedded /></div><div className="border rounded p-4 text-sm text-muted-foreground">Queue controls coming Phase2 — see <a href="/call_centers" className="underline">Call Centers</a> for queue config. History summary + agent status here in Phase2.</div></div>;
}
