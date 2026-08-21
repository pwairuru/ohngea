import * as React from "react";
import { apiFetch } from "../lib/api";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Card } from "../components/ui/card";

export function Gateways() {
  const [data, setData] = React.useState<any[]>([]);
  React.useEffect(() => { apiFetch("/gateways").then((r) => setData(r.data || [])).catch(() => {}); }, []);
  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Gateways</h1>
      <Card>
        <Table>
          <TableHeader><TableRow><TableHead>Gateway</TableHead><TableHead>Enabled</TableHead></TableRow></TableHeader>
          <TableBody>
            {data.map((g) => <TableRow key={g.gateway_uuid}><TableCell>{g.gateway}</TableCell><TableCell>{g.enabled ? "yes" : "no"}</TableCell></TableRow>)}
            {data.length===0 && <TableRow><TableCell colSpan={2} className="text-center">No gateways</TableCell></TableRow>}
          </TableBody>
        </Table>
      </Card>
    </div>
  );
}
