import * as React from "react";
import { Link, useLocation } from "react-router-dom";
import { ChevronRight } from "lucide-react";
import { findRoute } from "../../lib/navConfig";

export function PageHeader({ title, description, actions }: { title: string; description?: string; actions?: React.ReactNode }) {
  const loc = useLocation();
  const meta = findRoute(loc.pathname);
  return (
    <div className="flex flex-wrap items-end justify-between gap-2">
      <div>
        <nav className="flex items-center gap-1 text-xs text-muted-foreground">
          <Link to="/" className="hover:text-foreground">Home</Link>
          {meta?.group && (
            <>
              <ChevronRight className="h-3 w-3" />
              <span>{meta.group}</span>
            </>
          )}
          {meta && (
            <>
              <ChevronRight className="h-3 w-3" />
              <span className="text-foreground">{meta.label}</span>
            </>
          )}
        </nav>
        <h1 className="mt-1 text-xl font-semibold tracking-tight">{title}</h1>
        {description && <p className="mt-0.5 text-sm text-muted-foreground">{description}</p>}
      </div>
      {actions && <div className="flex flex-wrap gap-2">{actions}</div>}
    </div>
  );
}
