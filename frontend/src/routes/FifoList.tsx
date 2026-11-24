import * as React from "react";
import { apiFetch } from "../lib/api";
import { LiveTable } from "../components/LiveTable";
import { Button } from "../components/ui/button";
import { toast, toastError } from "../lib/toast";

function CallersPanel({ fifo, onClose }: { fifo: string; onClose: () => void }) {
  const [callers, setCallers] = React.useState<any[]>([]);
  const load = React.useCallback(() => {
    apiFetch(`/fifo_list/${encodeURIComponent(fifo)}/callers`)
      .then((r: any) => setCallers(r?.data ?? []))
      .catch(() => setCallers([]));
  }, [fifo]);
  React.useEffect(() => { load(); }, [load]);

  const remove = async (uuid: string) => {
    if (!confirm(`Remove caller ${uuid} from ${fifo}?`)) return;
    try {
      await apiFetch(`/fifo_list/${encodeURIComponent(fifo)}/callers/${encodeURIComponent(uuid)}`, { method: "DELETE" });
      toast("Caller removed", { variant: "success" });
      load();
    } catch (err) { toastError(err); }
  };

  return (
    <div className="border rounded p-3 space-y-2 bg-card">
      <div className="flex items-center justify-between">
        <span className="text-sm font-semibold">Callers in {fifo} ({callers.length})</span>
        <Button variant="ghost" size="sm" onClick={onClose}>Close</Button>
      </div>
      {callers.length === 0 && <div className="text-xs text-muted-foreground">No waiting callers</div>}
      {callers.map((c: any, i: number) => {
        const id = String(c.uuid ?? i);
        return (
          <div key={id} className="flex items-center justify-between gap-2 text-sm border rounded px-2 py-1">
            <span className="font-mono text-xs">{c.caller_id_number ?? id} · wait {c.wait_time ?? "—"}</span>
            {c.uuid && <Button variant="ghost" size="sm" className="text-destructive" onClick={() => remove(id)}>Remove</Button>}
          </div>
        );
      })}
    </div>
  );
}

export function FifoList() {
  const [openFifo, setOpenFifo] = React.useState<string | null>(null);

  return (
    <>
      <LiveTable
        title="FIFO List"
        description="Live FIFO consumers and callers"
        resource="/fifo_list"
        rowActions={(row: any) => (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setOpenFifo(String(row.fifo_name ?? ""))}
            disabled={!row.fifo_name}
          >
            Callers
          </Button>
        )}
      />
      {openFifo && (
        <div className="mt-3">
          <CallersPanel fifo={openFifo} onClose={() => setOpenFifo(null)} />
        </div>
      )}
    </>
  );
}
