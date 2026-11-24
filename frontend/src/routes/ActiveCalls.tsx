import { apiFetch } from "../lib/api";
import { LiveTable } from "../components/LiveTable";
import { Button } from "../components/ui/button";
import { toast, toastError } from "../lib/toast";

export function ActiveCalls() {
  const hangup = async (row: any, reload: () => void) => {
    const uid = row.call_uuid ?? row.uuid ?? row.id;
    if (!uid || !confirm(`Hangup call ${uid}?`)) return;
    try {
      await apiFetch(`/active_calls/${encodeURIComponent(String(uid))}/hangup`, { method: "POST" });
      toast("Hangup sent", { variant: "success" });
      reload();
    } catch (err) { toastError(err); }
  };

  return (
    <LiveTable
      title="Active Calls"
      description="Live switch channels via ESL"
      resource="/active_calls"
      rowActions={(row: any, reload: () => void) => (
        <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={() => hangup(row, reload)}>
          Hangup
        </Button>
      )}
    />
  );
}
