import * as React from "react";
import { getDeviceKey } from "../../context/webphoneStore";

export function DevicePicker({onOutputChange, onInputChange}:{onOutputChange?:(v:string)=>void; onInputChange?:(v:string)=>void}= {}){
  const [inputs,setInputs]=React.useState<MediaDeviceInfo[]>([]);
  const [outputs,setOutputs]=React.useState<MediaDeviceInfo[]>([]);
  const [outputId,setOutputId]=React.useState<string>("");
  const [inputId,setInputId]=React.useState<string>("");

  React.useEffect(()=>{
    let cancelled=false;
    (async()=>{
      try{
        try{ await navigator.mediaDevices.getUserMedia({audio:true}); }catch{}
        const ds=await navigator.mediaDevices.enumerateDevices();
        if(cancelled) return;
        setInputs(ds.filter(d=>d.kind==="audioinput"));
        setOutputs(ds.filter(d=>d.kind==="audiooutput"));
        try{
          const key=getDeviceKey();
          if(!key) return;
          const raw=localStorage.getItem(key);
          if(raw){
            const parsed=JSON.parse(raw);
            if(parsed.outputId){ setOutputId(parsed.outputId); try{ const el=document.getElementById("webphone-remote-audio") as HTMLAudioElement|null; if(el && typeof (el as any).setSinkId==="function") (el as any).setSinkId(parsed.outputId).catch(()=>{}); }catch{} }
            if(parsed.inputId) setInputId(parsed.inputId);
          }
        }catch{}
      }catch{}
    })();
    return ()=>{ cancelled=true; };
  },[]);

  function persist(next:{outputId?:string; inputId?:string}){
    try{
      const key=getDeviceKey();
      if(!key) return;
      const raw=localStorage.getItem(key);
      let cur:any={}; try{ if(raw) cur=JSON.parse(raw);}catch{}
      const merged={...cur, ...next};
      localStorage.setItem(key, JSON.stringify(merged));
    }catch{}
  }

  function handleOutputChange(e:React.ChangeEvent<HTMLSelectElement>){
    const v=e.target.value;
    setOutputId(v);
    persist({outputId:v});
    onOutputChange?.(v);
    try{
      const el=document.getElementById("webphone-remote-audio") as HTMLAudioElement|null;
      // @ts-ignore
      if(el && typeof el.setSinkId==="function") el.setSinkId(v).catch(()=>{});
    }catch{}
  }
  function handleInputChange(e:React.ChangeEvent<HTMLSelectElement>){
    const v=e.target.value;
    setInputId(v);
    persist({inputId:v});
    onInputChange?.(v);
  }

  return (
    <div data-testid="device-picker" className="flex flex-col gap-2">
      <label className="text-sm font-medium">Microphone</label>
      <select data-testid="device-input-select" value={inputId} onChange={handleInputChange} className="border rounded px-3 py-2 text-sm min-h-[44px]">
        <option value="">Default</option>
        {inputs.map(d=> <option key={d.deviceId} value={d.deviceId}>{d.label || `Mic ${d.deviceId.slice(0,5)}`}</option>)}
      </select>
      <label className="text-sm font-medium">Speaker</label>
      <select data-testid="device-output-select" value={outputId} onChange={handleOutputChange} className="border rounded px-3 py-2 text-sm min-h-[44px]">
        <option value="">Default</option>
        {outputs.map(d=> <option key={d.deviceId} value={d.deviceId}>{d.label || `Speaker ${d.deviceId.slice(0,5)}`}</option>)}
      </select>
    </div>
  );
}
