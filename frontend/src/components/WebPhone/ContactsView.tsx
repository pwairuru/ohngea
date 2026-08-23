import * as React from "react";
export function ContactsView({contacts, onDial, onToggleFav}:{contacts:any[]; onDial:(p:string)=>void; onToggleFav:(id:string)=>void}){
  const [q,setQ]=React.useState(""); const filtered=contacts.filter(c=> !q || c.primary.includes(q) || (c.name&&c.name.toLowerCase().includes(q.toLowerCase())));
  const favs=filtered.filter(c=>c.favorite), rest=filtered.filter(c=>!c.favorite);
  return <div><input data-testid="contacts-search" value={q} onChange={e=>setQ(e.target.value)} placeholder="Search extensions or users" className="w-full border rounded px-3 py-2 text-sm"/><div data-testid="contacts-list" className="mt-2 flex flex-col gap-1">{[...favs,...rest].map(c=><div key={c.id} className="flex justify-between border rounded px-3 py-2 text-sm"><span>{c.primary} — {c.name}</span><span className="flex gap-2"><button data-testid={`favorite-${c.id}`} onClick={()=>onToggleFav(c.id)} className="min-h-[44px] min-w-[44px]">{c.favorite?"★":"☆"}</button><button data-testid={`contact-dial-${c.primary}`} onClick={()=>onDial(c.primary)} className="min-h-[44px]">Call</button></span></div>)}</div></div>;
}
