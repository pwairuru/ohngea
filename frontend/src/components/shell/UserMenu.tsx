import * as React from "react";
import { useNavigate } from "react-router-dom";
import { apiFetch } from "../../lib/api";

export function UserMenu() {
  const nav = useNavigate();
  const [open, setOpen] = React.useState(false);
  const [me, setMe] = React.useState<{ username?: string; domain_id?: string }>({});
  const ref = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    apiFetch("/auth/me").then(setMe).catch(() => {});
  }, []);

  React.useEffect(() => {
    if (!open) return;
    const h = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", h);
    return () => document.removeEventListener("mousedown", h);
  }, [open]);

  const initial = (me.username ?? "U").charAt(0).toUpperCase();

  return (
    <div ref={ref} className="relative">
      <button
        onClick={() => setOpen((v) => !v)}
        aria-label="Account menu"
        className="flex h-8 w-8 items-center justify-center rounded-full bg-primary text-xs font-bold text-primary-foreground md:h-9 md:w-9 min-h-[44px] min-w-[44px] md:min-h-0 md:min-w-0"
      >
        {initial}
      </button>
      {open && (
        <div className="absolute right-0 top-full mt-2 w-56 rounded-md border bg-popover text-popover-foreground shadow-lg p-2 z-50">
          <div className="px-2 py-1.5">
            <div className="text-sm font-medium truncate">{me.username ?? "Signed in"}</div>
            {me.domain_id && <div className="text-xs text-muted-foreground truncate num">{me.domain_id}</div>}
          </div>
          <div className="my-1 border-t" />
          <button
            data-testid="logout-btn"
            onClick={() => {
              localStorage.removeItem("token");
              nav("/login");
            }}
            className="w-full rounded px-2 py-1.5 text-left text-sm hover:bg-accent min-h-[36px]"
          >
            Logout
          </button>
        </div>
      )}
    </div>
  );
}
