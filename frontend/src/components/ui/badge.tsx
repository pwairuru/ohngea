import * as React from "react";
import { cn } from "../../lib/utils";
export function Badge({ className, variant, ...props }: React.HTMLAttributes<HTMLDivElement> & { variant?: string }) {
  void variant;
  return <div className={cn("inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold bg-primary text-primary-foreground", className)} {...props} />;
}
