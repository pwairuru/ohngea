import * as React from "react";
import { apiFetch } from "../lib/api";
import { LiveTable } from "../components/LiveTable";
import { Button } from "../components/ui/button";
import { toast, toastError } from "../lib/toast";

const MEMBER_ACTIONS = ["mute", "unmute", "deaf", "undeaf", "kick"] as const;

function MembersPanel({ conf, onClose }: { conf: string; onClose: () => void }) {
  const [members, setMembers] = React.useState<any[]>([]);
  const load = React.useCallback(() => {
    apiFetch(`/active_conferences/${encodeURIComponent(conf)}/members`)
      .then((r: any) => setMembers(r?.data ?? []))
      .catch(() => setMembers([]));
  }, [conf]);
  React.useEffect(() => { load(); }, [load]);

  const act = async (member: string, action: string) => {
    try {
      await apiFetch(`/active_conferences/${encodeURIComponent(conf)}/members/${encodeURIComponent(member)}/${action}`, { method: "POST" });
      toast(`${action} ${member}`, { variant: "success" });
      load();
    } catch (err) { toastError(err); }
  };

  return (
    <div className="border rounded p-3 space-y-2 bg-card">
      <div className="flex items-center justify-between">
        <span className="text-sm font-semibold">Members of {conf} ({members.length})</span>
        <Button variant="ghost" size="sm" onClick={onClose}>Close</Button>
      </div>
      {members.length === 0 && <div className="text-xs text-muted-foreground">No members</div>}
      {members.map((m: any, i: number) => {
        const id = String(m.member_id ?? m.uuid ?? i);
        return (
          <div key={id} className="flex flex-wrap items-center justify-between gap-2 text-sm border rounded px-2 py-1">
            <span className="font-mono text-xs">{m.caller_id_number ?? id}{m.muted ? " (muted)" : ""}{m.deaf ? " (deaf)" : ""}</span>
            <span className="flex gap-1">
              {MEMBER_ACTIONS.map((a) => (
                <Button key={a} variant="ghost" size="sm" onClick={() => act(id, a)}>{a}</Button>
              ))}
            </span>
          </div>
        );
      })}
    </div>
  );
}

export function ActiveConferences() {
  const [openConf, setOpenConf] = React.useState<string | null>(null);

  return (
    <>
      <LiveTable
        title="Active Conferences"
        description="Live conference sessions via ESL"
        resource="/active_conferences"
        rowActions={(row: any) => (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setOpenConf(String(row.conference_name ?? row.name ?? ""))}
            disabled={!row.conference_name && !row.name}
          >
            Members
          </Button>
        )}
      />
      {openConf && (
        <div className="mt-3">
          <MembersPanel conf={openConf} onClose={() => setOpenConf(null)} />
        </div>
      )}
    </>
  );
}
