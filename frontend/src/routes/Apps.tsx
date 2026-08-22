import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Badge } from "../components/ui/badge";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";

type DomainApp = {
  app_name: string;
  enabled: boolean;
  category: string;
  depends_on: string[];
  description: string;
  source?: string;
};

function resolveDomainId(): string | null {
  const fromLs = typeof window !== "undefined" ? localStorage.getItem("domain_id") : null;
  if (fromLs) return fromLs;
  try {
    const token = typeof window !== "undefined" ? localStorage.getItem("token") : null;
    if (!token) return null;
    const payload = JSON.parse(atob(token.split(".")[1]));
    if (payload?.domain_id) return String(payload.domain_id);
    if (payload?.domainId) return String(payload.domainId);
    if (payload?.domain_uuid) return String(payload.domain_uuid);
  } catch {
    // ignore
  }
  return null;
}

export function Apps() {
  const [apps, setApps] = React.useState<DomainApp[]>([]);
  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);
  const [search, setSearch] = React.useState("");
  const [toggling, setToggling] = React.useState<string | null>(null);
  const [conflict, setConflict] = React.useState<{ app: DomainApp; message: string } | null>(null);

  const domainId = resolveDomainId();

  const load = React.useCallback(async () => {
    if (!domainId) {
      setError("No domain selected — login with domain");
      setLoading(false);
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const res: any = await apiFetch(`/domains/${domainId}/apps`);
      const data: DomainApp[] = Array.isArray(res) ? res : res?.data || [];
      setApps(data);
    } catch (e: any) {
      setError(e?.message || "Failed to load apps");
    } finally {
      setLoading(false);
    }
  }, [domainId]);

  React.useEffect(() => {
    load();
  }, [load]);

  const toggle = async (app: DomainApp, force = false) => {
    if (!domainId) return;
    const nextEnabled = !app.enabled;
    setToggling(app.app_name);
    try {
      const qs = `enabled=${nextEnabled}${force ? "&force=true" : ""}`;
      await apiFetch(`/domains/${domainId}/apps/${app.app_name}?${qs}`, { method: "PUT" });
      setApps((prev) => prev.map((a) => (a.app_name === app.app_name ? { ...a, enabled: nextEnabled } : a)));
      setConflict(null);
    } catch (e: any) {
      const msg = e?.message || "";
      if (msg.includes("409") || msg.includes("dependents")) {
        setConflict({ app, message: msg });
      } else {
        alert(msg);
      }
    } finally {
      setToggling(null);
    }
  };

  const filtered = React.useMemo(() => {
    if (!search.trim()) return apps;
    const q = search.toLowerCase();
    return apps.filter(
      (a) =>
        a.app_name.toLowerCase().includes(q) ||
        (a.description || "").toLowerCase().includes(q) ||
        (a.category || "").toLowerCase().includes(q),
    );
  }, [apps, search]);

  const grouped = React.useMemo(() => {
    const m = new Map<string, DomainApp[]>();
    for (const a of filtered) {
      const cat = a.category || "advanced";
      if (!m.has(cat)) m.set(cat, []);
      m.get(cat)!.push(a);
    }
    // sort categories and apps inside
    const sorted = Array.from(m.entries()).sort(([a], [b]) => a.localeCompare(b));
    for (const [, list] of sorted) list.sort((x, y) => x.app_name.localeCompare(y.app_name));
    return sorted;
  }, [filtered]);

  if (loading) {
    return <div className="p-4 text-sm text-muted-foreground">Loading apps…</div>;
  }
  if (error) {
    return <div className="p-4 text-sm text-red-500">{error}</div>;
  }

  return (
    <div className="p-4 space-y-4">
      <div className="flex flex-wrap justify-between items-center gap-2">
        <h1 className="text-2xl font-bold" data-testid="apps-title">Apps</h1>
        <div className="flex items-center gap-2">
          <Badge data-testid="apps-count">{filtered.length} / {apps.length}</Badge>
          <Button variant="ghost" onClick={load} data-testid="apps-refresh">Refresh</Button>
        </div>
      </div>

      <div className="flex gap-2">
        <Input
          placeholder="Search apps, category, description…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          data-testid="apps-search"
          className="max-w-md"
        />
      </div>

      {grouped.length === 0 && <div className="text-center text-muted-foreground py-8">No apps match “{search}”</div>}

      {grouped.map(([category, list]) => (
        <div key={category} className="space-y-2">
          <h2 className="text-sm font-semibold uppercase tracking-wide text-muted-foreground px-1">
            {category} <span className="font-normal normal-case">({list.length})</span>
          </h2>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {list.map((a) => (
              <Card key={a.app_name} className="flex flex-col justify-between" data-testid={`app-card-${a.app_name}`}>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm flex justify-between items-center gap-2">
                    <span className="font-mono truncate" title={a.app_name}>{a.app_name}</span>
                    <Badge className={a.enabled ? "bg-green-100 text-green-800 border-green-200" : "bg-muted text-muted-foreground"}>
                      {a.enabled ? "on" : "off"}
                    </Badge>
                  </CardTitle>
                  <p className="text-xs text-muted-foreground line-clamp-2 min-h-[2.5rem]">{a.description || "—"}</p>
                </CardHeader>
                <CardContent className="pt-0 space-y-2">
                  <div className="flex flex-wrap gap-1 text-xs">
                    <span className="rounded bg-muted px-2 py-0.5">{a.category}</span>
                    {a.depends_on && a.depends_on.length > 0 && (
                      <span className="rounded bg-amber-100 text-amber-800 px-2 py-0.5" title={`depends on ${a.depends_on.join(", ")}`}>
                        → {a.depends_on.join(", ")}
                      </span>
                    )}
                  </div>
                  <label className="flex items-center justify-between gap-2 cursor-pointer border rounded-lg px-3 py-2 hover:bg-muted/50">
                    <span className="text-sm font-medium">{a.enabled ? "Enabled" : "Disabled"}</span>
                    <input
                      type="checkbox"
                      role="switch"
                      aria-label={`Toggle ${a.app_name}`}
                      aria-checked={a.enabled}
                      checked={a.enabled}
                      disabled={toggling === a.app_name}
                      onChange={() => toggle(a)}
                      data-testid={`toggle-${a.app_name}`}
                      className="h-5 w-10 appearance-none rounded-full bg-muted relative cursor-pointer transition-colors checked:bg-primary after:absolute after:left-0.5 after:top-0.5 after:h-4 after:w-4 after:rounded-full after:bg-white after:transition-transform checked:after:translate-x-5 disabled:opacity-50"
                    />
                  </label>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      ))}

      <Dialog open={!!conflict} onOpenChange={(v) => !v && setConflict(null)}>
        <DialogHeader>
          <DialogTitle>Cannot disable {conflict?.app.app_name}</DialogTitle>
        </DialogHeader>
        {conflict && (
          <div className="space-y-3">
            <p className="text-sm text-muted-foreground whitespace-pre-wrap break-words">{conflict.message}</p>
            <p className="text-sm">Dependents still enabled. Disable them first or force.</p>
            <div className="flex gap-2 justify-end">
              <Button variant="ghost" onClick={() => setConflict(null)} data-testid="conflict-cancel">Cancel</Button>
              <Button
                data-testid="conflict-force"
                onClick={() => {
                  const app = conflict.app;
                  setConflict(null);
                  toggle(app, true);
                }}
              >
                Force disable
              </Button>
            </div>
          </div>
        )}
      </Dialog>
    </div>
  );
}

export default Apps;
