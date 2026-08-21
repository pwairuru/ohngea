import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";

export function Users() {
  const [me, setMe] = React.useState<any>(null);
  React.useEffect(() => { apiFetch("/auth/me").then(setMe).catch(() => {}); }, []);
  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Users & Groups</h1>
      <Card><CardHeader><CardTitle>Me</CardTitle></CardHeader><CardContent>{me ? <div className="font-mono text-sm">{me.username} — {me.user_uuid} — domain {me.domain_id}</div> : <div className="text-muted-foreground">Login to see user</div>}</CardContent></Card>
      <Card><CardHeader><CardTitle>RBAC</CardTitle></CardHeader><CardContent className="text-sm text-muted-foreground">Groups: superadmin. Permissions: domain_view, extension_add, dialplan_view etc. RLS via JWT domain_id.</CardContent></Card>
    </div>
  );
}
