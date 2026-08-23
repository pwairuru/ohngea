import * as React from "react";
import { cn } from "../../lib/utils";
export function Tabs({ defaultValue, value: controlledValue, onValueChange, children, className }: { defaultValue?: string; value?: string; onValueChange?: (v:string)=>void; children: React.ReactNode; className?: string }) {
  const [internal, setInternal] = React.useState(defaultValue ?? controlledValue ?? "");
  const isControlled = controlledValue !== undefined;
  const value = isControlled ? controlledValue : internal;
  const setValue = (v:string)=>{ if(!isControlled) setInternal(v); onValueChange?.(v); };
  return <div className={cn("", className)}>{React.Children.map(children, (c: any) => c ? React.cloneElement(c, { value, setValue }) : c)}</div>;
}
export function TabsList({ children, value, setValue }: any) {
  return <div role="tablist" className="inline-flex h-10 items-center rounded-md bg-muted p-1">{React.Children.map(children, (c: any) => c ? React.cloneElement(c, { value, setValue }) : c)}</div>;
}
export function TabsTrigger({ value, setValue, triggerValue, children }: any) {
  const active = value === triggerValue;
  return <button role="tab" aria-selected={active} data-state={active?"active":"inactive"} onClick={() => setValue(triggerValue)} className={`px-3 py-1.5 text-sm font-medium rounded-sm min-h-[44px] ${active ? "bg-background shadow" : ""}`}>{children}</button>;
}
export function TabsContent({ value, contentValue, children }: any) {
  if (value !== contentValue) return null;
  return <div role="tabpanel" className="mt-4 ring-offset-background">{children}</div>;
}
