import * as React from "react";
import { apiFetch } from "../lib/api";
import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { FormDialog } from "../components/FormDialog";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";

export function Fax() {
  const [sendRow, setSendRow] = React.useState<any | null>(null);
  const [number, setNumber] = React.useState("");
  const [reloadKey, setReloadKey] = React.useState(0);
  const [uploadRow, setUploadRow] = React.useState<any | null>(null);
  const [fileName, setFileName] = React.useState("");
  const [fileB64, setFileB64] = React.useState("");

  const { run: send, pending } = useAsyncAction(async () => {
    try {
      const res: any = await apiFetch(`/fax/${sendRow.fax_uuid}/send`, {
        method: "POST",
        body: JSON.stringify({ fax_number: number }),
      });
      toast(`Fax queued to ${number}`, { variant: "success" });
      setSendRow(null);
      setNumber("");
      setReloadKey((k) => k + 1);
    } catch (err) { toastError(err); }
  });

  const { run: upload, pending: uploading } = useAsyncAction(async () => {
    try {
      await apiFetch(`/fax/${uploadRow.fax_uuid}/files`, {
        method: "POST",
        body: JSON.stringify({ file_name: fileName || "document.tiff", file_base64: fileB64 }),
      });
      toast("File uploaded", { variant: "success" });
      setUploadRow(null);
      setFileName("");
      setFileB64("");
      setReloadKey((k) => k + 1);
    } catch (err) { toastError(err); }
  });

  const pickFile = (f: File | undefined) => {
    if (!f) return;
    setFileName(f.name);
    const r = new FileReader();
    r.onload = () => {
      const s = String(r.result ?? "");
      setFileB64(s.includes(",") ? s.split(",")[1] : s);
    };
    r.readAsDataURL(f);
  };

  return (
    <>
      <div key={reloadKey}>
        <FullCrudPage
          config={CRUD_CONFIGS.fax}
          extraRowActions={(row: any) => (
            <>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => { setSendRow(row); setNumber(""); }}
              >
                Send
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => { setUploadRow(row); setFileName(""); setFileB64(""); }}
              >
                Upload
              </Button>
            </>
          )}
        />
      </div>
      <FormDialog
        open={!!sendRow}
        onOpenChange={(v) => { if (!v) setSendRow(null); }}
        title={`Send fax ${sendRow?.fax_extension ?? ""}`}
        onSubmit={send}
        pending={pending}
        submitLabel="Queue Send"
      >
        <div><Label>Destination number *</Label><Input value={number} onChange={(e) => setNumber(e.target.value)} placeholder="15551234567" /></div>
      </FormDialog>
      <FormDialog
        open={!!uploadRow}
        onOpenChange={(v) => { if (!v) setUploadRow(null); }}
        title={`Upload file ${uploadRow?.fax_extension ?? ""}`}
        onSubmit={upload}
        pending={uploading}
        submitLabel="Upload"
        disabled={!fileB64}
      >
        <div><Label>File *</Label><Input type="file" accept=".tiff,.tif,.pdf" onChange={(e) => pickFile(e.target.files?.[0])} /></div>
        {fileName && <div className="text-xs text-muted-foreground">{fileName} ready</div>}
      </FormDialog>
    </>
  );
}
