import * as React from "react";
import { PackageOpen } from "lucide-react";

export function EmptyState({ icon: Icon = PackageOpen, title, description, action }: { icon?: React.ComponentType<{ className?: string }>; title: string; description?: string; action?: React.ReactNode }) {
  return (
    <div data-testid="empty-state" className="flex flex-col items-center justify-center py-16 px-4 text-center">
      <Icon className="h-10 w-10 text-muted-foreground/50" />
      <div className="mt-3 text-sm font-medium">{title}</div>
      {description && <div className="mt-1 text-xs text-muted-foreground">{description}</div>}
      {action && <div className="mt-4">{action}</div>}
    </div>
  );
}
