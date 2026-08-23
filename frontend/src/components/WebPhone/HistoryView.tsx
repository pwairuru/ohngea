import * as React from "react";
export function HistoryView({history, contacts, onRedial, onClear}:{history:any[]; contacts?:any[]; onRedial:(p:string)=>void; onClear:()=>void}){
  const contactsMap=React.useMemo(()=>{
    if(!contacts?.length) return {} as Record<string,string>;
    const m:Record<string,string>={};
    for(const c of contacts){ if(c.primary) m[c.primary]=c.name || c.primary; }
    return m;
  },[contacts]);
  if(!history.length) return <div className="py-8 text-center text-sm text-muted-foreground">No recent calls</div>;
  return <div data-testid="history-list" className="flex flex-col gap-1">{history.slice().reverse().map((r:any)=>{ const name = r.peerName || contactsMap[r.peer] || r.peer; return <div data-testid={`history-item-${r.id}`} key={r.id} className="flex items-center justify-between border rounded px-3 py-2 text-sm"><span>{r.direction==="in"?"↙":"↗"} {name} • {r.disposition} • {r.duration}s</span><button data-testid={`history-redial-${r.peer}`} onClick={()=>onRedial(r.peer)} className="underline min-h-[44px] min-w-[44px]">Redial</button></div>; })}<button onClick={onClear} className="text-xs underline mt-2 min-h-[44px]">Clear history</button></div>;
}
