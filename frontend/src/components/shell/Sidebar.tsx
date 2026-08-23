import * as React from "react";
import { Link, useLocation } from "react-router-dom";
import { ChevronDown, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { cn } from "../../lib/utils";
import { primaryNav, advancedNav, type NavGroup } from "../../lib/navConfig";

function isActive(loc: { pathname: string }, to: string) {
  return to === "/" ? loc.pathname === "/" : loc.pathname.startsWith(to);
}

function NavLinkRow({ to, label, icon: Icon, active, rail, group }: { to: string; label: string; icon: React.ComponentType<{ className?: string }>; active: boolean; rail?: boolean; group?: string }) {
  return (
    <Link
      to={to}
      title={rail ? `${group ? group + " · " : ""}${label}` : undefined}
      data-active={active || undefined}
      className={cn(
        "group relative flex items-center gap-2.5 rounded-md px-3 py-1.5 text-sm min-h-[36px] outline-none focus-visible:ring-2 focus-visible:ring-ring",
        active ? "bg-sidebar-accent text-sidebar-accent-foreground font-medium" : "text-sidebar-foreground/80 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
        rail && "md:justify-center md:px-0"
      )}
    >
      {active && <span className="absolute left-0 top-1.5 bottom-1.5 w-0.5 rounded-full bg-primary" />}
      <Icon className="h-4 w-4 shrink-0" />
      {!rail && <span className="truncate">{label}</span>}
    </Link>
  );
}

function NavSection({ loc, rail, openGroups, toggleGroup }: { loc: { pathname: string }; rail: boolean; openGroups: string[]; toggleGroup: (g: string) => void }) {
  return (
    <>
      {advancedNav.map((g: NavGroup) => {
        const open = rail || openGroups.includes(g.label);
        return (
          <div key={g.label}>
            {!rail && (
              <button onClick={() => toggleGroup(g.label)} className="flex w-full items-center justify-between rounded px-3 py-2 text-xs font-medium uppercase tracking-wider text-muted-foreground hover:text-foreground" aria-expanded={open}>
                {g.label}
                <ChevronDown className={cn("h-3.5 w-3.5 transition-transform", open && "rotate-180")} />
              </button>
            )}
            {open && (
              <div className="flex flex-col gap-0.5 py-1">
                {g.items.map((it) => (
                  <NavLinkRow key={it.to} {...it} active={isActive(loc, it.to)} rail={rail} group={g.label} />
                ))}
              </div>
            )}
          </div>
        );
      })}
    </>
  );
}

export function Sidebar() {
  const loc = useLocation();
  const [rail, setRail] = React.useState(() => localStorage.getItem("nav-rail") === "1");
  const [openGroups, setOpenGroups] = React.useState<string[]>(() => {
    try {
      return JSON.parse(localStorage.getItem("nav-open-groups") || '["Call Routing"]');
    } catch {
      return ["Call Routing"];
    }
  });
  const toggleRail = () => {
    setRail((v) => {
      localStorage.setItem("nav-rail", v ? "0" : "1");
      return !v;
    });
  };
  const toggleGroup = (label: string) => {
    setOpenGroups((prev) => {
      const next = prev.includes(label) ? prev.filter((g) => g !== label) : [...prev, label];
      localStorage.setItem("nav-open-groups", JSON.stringify(next));
      return next;
    });
  };
  return (
    <aside
      data-testid="sidebar"
      className={cn(
        "sticky top-14 hidden h-[calc(100vh-3.5rem)] shrink-0 flex-col overflow-y-auto border-r border-sidebar-border bg-sidebar p-3 md:flex",
        rail ? "w-12" : "w-56"
      )}
    >
      <div className="flex flex-col gap-0.5">
        {primaryNav.map((it) => (
          <NavLinkRow key={it.to} {...it} active={isActive(loc, it.to)} rail={rail} />
        ))}
      </div>
      {!rail && (
        <div className="mb-1 mt-4 flex items-center justify-between px-3">
          <span className="text-xs font-medium uppercase tracking-wider text-muted-foreground">Advanced</span>
          <button data-testid="sidebar-rail-toggle" aria-label="Collapse sidebar" onClick={toggleRail} className="text-muted-foreground hover:text-foreground">
            <PanelLeftClose className="h-4 w-4" />
          </button>
        </div>
      )}
      {rail && (
        <div className="my-3 border-t border-sidebar-border pt-3">
          <button data-testid="sidebar-rail-toggle" aria-label="Expand sidebar" onClick={toggleRail} className="flex w-full justify-center py-1 text-muted-foreground hover:text-foreground">
            <PanelLeftOpen className="h-4 w-4" />
          </button>
        </div>
      )}
      {!rail && <NavSection loc={loc} rail={rail} openGroups={openGroups} toggleGroup={toggleGroup} />}
      <div className="mt-auto pt-4 text-[11px] text-muted-foreground">
        {!rail && (
          <>
            <div>PBX v1.0.0</div>
            <div>REST /api/v1</div>
          </>
        )}
      </div>
    </aside>
  );
}
