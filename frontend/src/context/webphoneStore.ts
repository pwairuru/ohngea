export type CallRecord = { id:string; direction:"in"|"out"; peer:string; peerName:string|null; startedAt:string; duration:number; disposition:"answered"|"missed"|"failed"; domain_id:string; user_uuid:string };
export type Contact = { id:string; kind:"extension"|"user"; primary:string; name:string; favorite:boolean; domain_id:string };
export const historyKey = (d:string,u:string)=>`webphone:history:${d}:${u}`;
export const deviceKey = (d:string,u:string)=>`webphone:devices:${d}:${u}`;
export function getDeviceKey(): string | null {
  try{
    const t=localStorage.getItem("token");
    if(!t) return null;
    const payload=JSON.parse(atob(t.split(".")[1]));
    const domain=payload.domain_id;
    const user=payload.sub;
    if(typeof domain!=="string" || typeof user!=="string" || !domain || !user) return null;
    return deviceKey(domain, user);
  }catch{ return null; }
}
export const initial = { history: [] as CallRecord[], contacts: [] as Contact[] };
export function reducer(state:any, action:any){
  if(action.type==="HISTORY_APPEND"){
    let h=[...state.history, action.rec]; if(h.length>100) h=h.slice(h.length-100); return {...state, history:h};
  }
  if(action.type==="HISTORY_CLEAR") return {...state, history:[]};
  if(action.type==="HISTORY_LOAD") return {...state, history:action.history};
  return state;
}
