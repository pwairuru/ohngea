import * as React from "react";
import { apiFetch } from "../lib/api";
import { LiveTable } from "../components/LiveTable";
import { Button } from "../components/ui/button";
import { toast, toastError } from "../lib/toast";

export function Registrations() {
  const act = async (row: any, action: "unregister" | "reboot" | "provision", reload: () => void) => {
    const id = row.sip_username ?? row.user ?? row.uid ?? row.id;
    if (!id) return;
    try {
      await apiFetch(`/registrations/${encodeURIComponent(String(id))}/${action}`, { method: "POST" });
      toast(`${action} sent to ${id}`, { variant: "success" });
      reload();
    } catch (err) { toastError(err); }
  };

  return (
    <LiveTable
      title="Registrations"
      description="SIP registrations via ESL with device actions"
      resource="/registrations"
      rowActions={(row: any, reload: () => void) => (
        <div className="flex justify-end gap-1">
          <Button variant="ghost" size="sm" onClick={() => act(row, "provision", reload)}>Provision</Button>
          <Button variant="ghost" size="sm" onClick={() => act(row, "reboot", reload)}>Reboot</Button>
          <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={() => act(row, "unregister", reload)}>Unregister</Button>
        </div>
      )}
    />
  );
}
