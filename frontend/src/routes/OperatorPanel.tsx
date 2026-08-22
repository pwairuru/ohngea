import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Badge } from "../components/ui/badge";
export function OperatorPanel() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const load = () => apiFetch("/operator_panel").then((r:any)=>{setData(r.data||[]); setTotal(r.total??r.data?.length??0)}).catch(()=>{});
  React.useEffect(()=>{load()},[]);
  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Operator Panel</h1><Badge>{total} total</Badge></div>
      <Card>
        <CardHeader><CardTitle>Operator Panel</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>Name / ID</TableHead><TableHead>Details</TableHead></TableRow></TableHeader>
              <TableBody>
                {data.map((d:any,i:number)=><TableRow key={i}><TableCell className="font-mono text-xs">{JSON.stringify(d).slice(0,80)}</TableCell><TableCell className="text-xs text-muted-foreground">{d._links ? "•" : ""}</TableCell></TableRow>)}
                {data.length===0 && <TableRow><TableCell colSpan={2} className="text-center text-muted-foreground py-8">No operator panel — create via API POST /api/v1/operator_panel</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((d:any,i:number)=><Card key={i} className="p-3"><div className="font-mono text-xs break-all">{JSON.stringify(d).slice(0,120)}</div></Card>)}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No operator panel</div>}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
