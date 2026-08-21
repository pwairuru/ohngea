import * as React from "react";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Badge } from "../../components/ui/badge";
export default function Ring_groupsPlugin() {
  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Ring_groups <Badge>plugin</Badge></h1>
      <Card><CardHeader><CardTitle>ring_groups per-tenant plugin</CardTitle></CardHeader><CardContent className="text-sm text-muted-foreground">This is a lazy-loaded plugin UI for ring_groups — toggle per domain via /api/v1/domains/{'{id}'}/plugins. License MIT, enabled via domain_plugins.</CardContent></Card>
      <Card><CardContent className="pt-4"><div className="text-xs font-mono">manifest: ring_groups v1.0.0 MIT</div></CardContent></Card>
    </div>
  );
}
