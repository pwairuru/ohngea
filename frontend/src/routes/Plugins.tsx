import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Badge } from "../components/ui/badge";

export function Plugins() {
  const [plugins, setPlugins] = React.useState<any[]>([]);
  const [domains, setDomains] = React.useState<any[]>([]);
  const [selectedDomain, setSelectedDomain] = React.useState<string>("");
  const [domainPlugins, setDomainPlugins] = React.useState<any[]>([]);

  React.useEffect(() => {
    apiFetch("/plugins").then((r) => setPlugins(r.data || r || [])).catch(() => {
      // fallback mock if API not ready
      setPlugins([
        { name: "call_center", license: "MIT", description: "Queue wallboard", category: "advanced" },
        { name: "fax", license: "MIT", description: "Fax queue", category: "advanced" },
      ]);
    });
    apiFetch("/domains").then((r) => {
      const list = Array.isArray(r) ? r : r.data || [];
      setDomains(list);
      if (list[0]) setSelectedDomain(list[0].domain_id);
    }).catch(() => {});
  }, []);

  React.useEffect(() => {
    if (!selectedDomain) return;
    apiFetch(`/domains/${selectedDomain}/plugins`).then((r) => setDomainPlugins(r || [])).catch(() => {});
  }, [selectedDomain]);

  const toggle = async (name: string, enabled: boolean) => {
    if (!selectedDomain) return alert("Pick domain first");
    await apiFetch(`/domains/${selectedDomain}/plugins/${name}?enabled=${!enabled}`, { method: "PUT" });
    const updated = await apiFetch(`/domains/${selectedDomain}/plugins`);
    setDomainPlugins(updated || []);
  };

  const isEnabled = (name: string) => {
    const found = domainPlugins.find((p) => p.plugin_name === name);
    return found ? found.enabled : true; // default enabled
  };

  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Plugins Marketplace</h1>
      <Card>
        <CardHeader><CardTitle>Per-tenant toggleable — license gate MIT/Apache/BSD/MPL only</CardTitle></CardHeader>
        <CardContent className="space-y-3">
          <div className="flex gap-2 items-center">
            <span className="text-sm">Tenant:</span>
            <select value={selectedDomain} onChange={(e) => setSelectedDomain(e.target.value)} className="border rounded px-3 py-2 flex-1 max-w-md min-h-[44px]">
              <option value="">Select domain</option>
              {domains.map((d) => <option key={d.domain_id} value={d.domain_id}>{d.domain_name}</option>)}
            </select>
          </div>
          <div className="text-xs text-muted-foreground">Plugins scanned from <code>plugins/*/manifest.toml</code> — GPL rejected at load. Disable hides route (403) and lazy UI.</div>
        </CardContent>
      </Card>

      <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
        {plugins.map((p) => (
          <Card key={p.name} className="p-4 flex flex-col">
            <div className="flex justify-between items-start">
              <div className="font-bold capitalize">{p.name.replace("_", " ")}</div>
              <Badge variant="outline">{p.license}</Badge>
            </div>
            <div className="text-xs text-muted-foreground mt-1">{p.description}</div>
            <div className="text-xs mt-2">routes: <code>{(p.routes || []).join(", ") || `/api/v1/${p.name}`}</code></div>
            <div className="text-xs">category: {p.category}</div>
            <div className="mt-3 flex gap-2">
              <Badge className={isEnabled(p.name) ? "bg-green-600" : "bg-muted text-muted-foreground"}>{isEnabled(p.name) ? "enabled" : "disabled"}</Badge>
              <Button variant="outline" className="h-8 flex-1" onClick={() => toggle(p.name, isEnabled(p.name))}>{isEnabled(p.name) ? "Disable" : "Enable"}</Button>
              <Button className="h-8" onClick={() => window.location.assign(`/api/v1/${p.name}`)}>Try API</Button>
            </div>
            <div className="mt-2 text-xs">
              <span className="text-muted-foreground">UI:</span> <code>frontend/src/plugins/{p.name}/App.tsx</code> lazy
            </div>
          </Card>
        ))}
        {plugins.length===0 && <Card className="p-8 text-center text-muted-foreground col-span-full">No plugins found — check plugins/ dir</Card>}
      </div>

      <Card>
        <CardHeader><CardTitle>How to create a plugin</CardTitle></CardHeader>
        <CardContent className="text-sm space-y-2">
          <div>1. <code>mkdir plugins/my_plugin && echo 'name="my_plugin"\nlicense="MIT"' &gt; plugins/my_plugin/manifest.toml</code></div>
          <div>2. Add <code>plugins/my_plugin/router.py</code> with <code>router = APIRouter(prefix="/api/v1/my_plugin")</code></div>
          <div>3. Add frontend <code>frontend/src/plugins/my_plugin/App.tsx</code> lazy chunk</div>
          <div>4. Restart API — registry validates license, auto-mounts router, inserts permissions</div>
          <div className="text-xs text-muted-foreground">See <code>docs/plugins.md</code> for full spec.</div>
        </CardContent>
      </Card>
    </div>
  );
}
