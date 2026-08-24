import * as React from "react";
import { Link, useLocation } from "react-router-dom";
import { ChevronDown } from "lucide-react";
import { cn } from "../../lib/utils";
import { primaryNav, advancedNav } from "../../lib/navConfig";
import { Sheet } from "../ui/sheet";

function isActive(pathname: string, to: string) {
  return to === "/" ? pathname === "/" : pathname.startsWith(to);
}

export function MobileDrawer({ open, onClose }: { open: boolean; onClose: () => void }) {
  const loc = useLocation();
  return (
    <Sheet open={open} onOpenChange={(v) => !v && onClose()} side="left">
      <div data-testid="mobile-drawer" className="flex h-full flex-col">
        <div className="mb-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <img src="/logo.svg" alt="Ohngea PBX" className="h-6 w-6 text-foreground" style={{ color: "currentColor" }} />
            <span className="font-bold">Ohngea PBX</span>
          </div>
          <button
            onClick={onClose}
            aria-label="Close menu"
            className="inline-flex h-10 w-10 items-center justify-center rounded-md border min-h-[44px] min-w-[44px]"
          >
            ✕
          </button>
        </div>
        <div className="flex flex-col gap-1">
          {primaryNav.map((it) => (
            <Link
              key={it.to}
              to={it.to}
              onClick={onClose}
              className={cn(
                "flex items-center gap-2.5 rounded-md px-3 py-3 text-base min-h-[44px]",
                isActive(loc.pathname, it.to) ? "bg-primary text-primary-foreground" : "hover:bg-accent border"
              )}
            >
              <it.icon className="h-4 w-4" />
              {it.label}
            </Link>
          ))}
        </div>
        <div className="mt-4 flex-1 overflow-y-auto border-t pt-4">
          {advancedNav.map((g) => (
            <details key={g.label} className="py-1" open={g.items.some((i) => isActive(loc.pathname, i.to))}>
              <summary className="flex cursor-pointer list-none items-center justify-between rounded px-3 py-2.5 text-sm font-medium min-h-[44px]">
                {g.label}
                <ChevronDown className="h-4 w-4 text-muted-foreground" />
              </summary>
              <div className="flex flex-col gap-1 pb-2 pl-2">
                {g.items.map((it) => (
                  <Link
                    key={it.to}
                    to={it.to}
                    onClick={onClose}
                    className={cn(
                      "flex items-center gap-2.5 rounded px-3 py-3 text-base min-h-[44px]",
                      isActive(loc.pathname, it.to) ? "bg-accent font-medium" : "hover:bg-accent"
                    )}
                  >
                    <it.icon className="h-4 w-4" />
                    {it.label}
                  </Link>
                ))}
              </div>
            </details>
          ))}
        </div>
      </div>
    </Sheet>
  );
}
