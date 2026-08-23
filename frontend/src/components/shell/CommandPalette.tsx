import * as React from "react";
import { useNavigate } from "react-router-dom";
import { Search, CornerDownLeft, Phone } from "lucide-react";
import { cn } from "../../lib/utils";
import { flatNav } from "../../lib/navConfig";
import { useTheme } from "../theme/ThemeProvider";

type Entry = { to?: string; label: string; icon?: React.ComponentType<{ className?: string }>; run?: () => void };

export function CommandPalette() {
  const [open, setOpen] = React.useState(false);
  const [q, setQ] = React.useState("");
  const [active, setActive] = React.useState(0);
  const nav = useNavigate();
  const { setTheme } = useTheme();

  React.useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen((v) => !v);
        setQ("");
        setActive(0);
      }
      if (e.key === "Escape") setOpen(false);
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, []);

  const entries = React.useMemo<Entry[]>(
    () => [
      ...flatNav.map((i) => ({ to: i.to, label: i.label, icon: i.icon })),
      { to: "/extensions", label: "New Extension", icon: Phone },
      { label: "Toggle theme", run: () => setTheme(document.documentElement.classList.contains("dark") ? "light" : "dark") },
    ],
    [setTheme]
  );

  const results = React.useMemo(() => {
    const needle = q.trim().toLowerCase();
    if (!needle) return entries.slice(0, 12);
    return entries.filter((e) => e.label.toLowerCase().includes(needle)).slice(0, 12);
  }, [entries, q]);

  if (!open) return null;

  const choose = (e: Entry | undefined) => {
    if (!e) return;
    setOpen(false);
    if (e.to) nav(e.to);
    e.run?.();
  };

  return (
    <div className="fixed inset-0 z-[60] flex items-start justify-center pt-[15vh]" onKeyDown={(e) => {
      if (e.key === "ArrowDown") { e.preventDefault(); setActive((a) => Math.min(a + 1, results.length - 1)); }
      if (e.key === "ArrowUp") { e.preventDefault(); setActive((a) => Math.max(a - 1, 0)); }
      if (e.key === "Enter") { e.preventDefault(); choose(results[active]); }
    }}>
      <div className="absolute inset-0 bg-black/50" onClick={() => setOpen(false)} />
      <div data-testid="command-palette" role="listbox" className="relative w-full max-w-lg mx-4 rounded-lg border bg-popover text-popover-foreground shadow-xl overflow-hidden">
        <div className="flex items-center gap-2 border-b px-3">
          <Search className="h-4 w-4 text-muted-foreground" />
          <input
            data-testid="command-input"
            autoFocus
            value={q}
            onChange={(e) => { setQ(e.target.value); setActive(0); }}
            placeholder="Search pages…"
            className="h-11 w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground"
          />
          <kbd className="hidden sm:inline-flex items-center gap-0.5 rounded border px-1.5 text-[10px] text-muted-foreground"><CornerDownLeft className="h-3 w-3" /> enter</kbd>
        </div>
        <div className="max-h-80 overflow-auto p-1">
          {results.map((e, idx) => (
            <button
              key={`${e.to}-${e.label}`}
              role="option"
              aria-selected={idx === active}
              onMouseEnter={() => setActive(idx)}
              onClick={() => choose(e)}
              className={cn(
                "flex w-full items-center gap-2.5 rounded-md px-3 py-2 text-sm text-left",
                idx === active ? "bg-accent text-accent-foreground" : "hover:bg-accent/50"
              )}
            >
              {e.icon ? <e.icon className="h-4 w-4 text-muted-foreground" /> : <span className="h-4 w-4" />}
              {e.label}
            </button>
          ))}
          {!results.length && <div className="px-3 py-6 text-center text-sm text-muted-foreground">No matches</div>}
        </div>
      </div>
    </div>
  );
}
