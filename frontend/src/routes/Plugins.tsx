import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Badge } from "../components/ui/badge";

export function Plugins() {
  const [plugins, setPlugins] = React.useState<any[]>([]);
  React.useEffect(() => {
    apiFetch("/plugins").then(setPlugins).catch(() => setPlugins([{ name: "example", license: "MIT", enabled: true }]));
  }, []);
  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Plugins Marketplace</h1>
      <Card>
        <CardHeader><CardTitle>Per-tenant toggleable — license gate MIT/Apache/BSD/MPL</CardTitle></CardHeader>
        <CardContent className="grid gap-3 md:grid-cols-2">
          {plugins.map((p) => (
            <Card key={p.name} className="p-4">
              <div className="font-bold">{p.name}</div>
              <div className="text-xs text-muted-foreground">license: {p.license}</div>
              <div className="mt-2 flex gap-2"><Badge>{p.enabled ? "enabled" : "disabled"}</Badge><Button variant="outline" className="h-8">Toggle</Button></div>
            </Card>
          ))}
          {plugins.length===0 && <div className="text-muted-foreground">No plugins</div>}
        </CardContent>
      </Card>
    </div>
  );
}
