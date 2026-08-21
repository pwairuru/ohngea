import * as React from "react";
import { cn } from "../../lib/utils";
export function Tabs({ defaultValue, children, className }: { defaultValue: string; children: React.ReactNode; className?: string }) {
  const [value, setValue] = React.useState(defaultValue);
  return <div className={cn("", className)}>{React.Children.map(children, (c: any) => React.cloneElement(c, { value, setValue }))}</div>;
}
export function TabsList({ children, value, setValue }: any) {
  return <div className="inline-flex h-10 items-center rounded-md bg-muted p-1">{React.Children.map(children, (c: any) => React.cloneElement(c, { value, setValue }))}</div>;
}
export function TabsTrigger({ value, setValue, triggerValue, children }: any) {
  const active = value === triggerValue;
  return <button onClick={() => setValue(triggerValue)} className={`px-3 py-1.5 text-sm font-medium rounded-sm min-h-[44px] ${active ? "bg-background shadow" : ""}`}>{children}</button>;
}
export function TabsContent({ value, contentValue, children }: any) {
  if (value !== contentValue) return null;
  return <div className="mt-4 ring-offset-background">{children}</div>;
}
