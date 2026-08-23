export type CallRecord = { id:string; direction:"in"|"out"; peer:string; peerName:string|null; startedAt:string; duration:number; disposition:"answered"|"missed"|"failed"; domain_id:string; user_uuid:string };
export type Contact = { id:string; kind:"extension"|"user"; primary:string; name:string; favorite:boolean; domain_id:string };
export const historyKey = (d:string,u:string)=>`webphone:history:${d}:${u}`;
export const initial = { history: [] as CallRecord[], contacts: [] as Contact[] };
export function reducer(state:any, action:any){
  if(action.type==="HISTORY_APPEND"){
    let h=[...state.history, action.rec]; if(h.length>100) h=h.slice(h.length-100); return {...state, history:h};
  }
  if(action.type==="HISTORY_CLEAR") return {...state, history:[]};
  if(action.type==="HISTORY_LOAD") return {...state, history:action.history};
  return state;
}
