import * as React from "react";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";

export default function CallCenterPlugin() {
  const [queues] = React.useState([
    { name: "Support", waiting: 5, agents: 3, avgWait: "01:23" },
    { name: "Sales", waiting: 2, agents: 4, avgWait: "00:45" },
  ]);
  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Call Center <Badge>plugin</Badge> <Badge variant="outline">MIT</Badge></h1>
      <div className="grid gap-4 md:grid-cols-3">
        {queues.map((q) => (
          <Card key={q.name}><CardHeader><CardTitle>{q.name}</CardTitle></CardHeader><CardContent>
            <div className="text-3xl font-bold">{q.waiting} waiting</div>
            <div className="text-sm text-muted-foreground">{q.agents} agents • avg {q.avgWait}</div>
            <Button variant="outline" className="mt-2 w-full">Wallboard</Button>
          </CardContent></Card>
        ))}
      </div>
      <Card><CardHeader><CardTitle>AMEYO wallboard KPI grid</CardTitle></CardHeader><CardContent className="text-sm text-muted-foreground">Queue wallboard — agents, waiting calls, avg wait, SLA — per tenant, data from `call_center` plugin `GET /api/v1/call_center` partitioned by domain_id.</CardContent></Card>
    </div>
  );
}
