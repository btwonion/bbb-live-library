import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Download, FileText, CheckCircle, AlertCircle } from "lucide-react";
import { toast } from "sonner";
import { triggerBbbImport } from "@/api/import";
import { Button } from "@/components/ui/button";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import type { ImportResult } from "@/api/types";

export default function SettingsPage() {
  useDocumentTitle("Settings");

  const [importResult, setImportResult] = useState<ImportResult | null>(null);

  const importMutation = useMutation({
    mutationFn: triggerBbbImport,
    onSuccess: (data) => {
      setImportResult(data);
      toast.success(`Import complete: ${data.imported} imported, ${data.skipped} skipped`);
    },
  });

  return (
    <div className="flex flex-col gap-8">
      <h1 className="text-2xl font-bold">Settings</h1>

      {/* BBB Import */}
      <section className="flex flex-col gap-4 rounded-lg border p-6">
        <div className="flex items-center gap-3">
          <Download className="size-5 text-muted-foreground" />
          <h2 className="text-lg font-semibold">BBB Import</h2>
        </div>
        <p className="text-sm text-muted-foreground">
          Manually trigger a bulk import of recordings from your BigBlueButton
          server. This fetches all published recordings and imports any that
          haven&apos;t been imported yet.
        </p>
        <div className="flex items-center gap-4">
          <Button
            onClick={() => {
              setImportResult(null);
              importMutation.mutate();
            }}
            disabled={importMutation.isPending}
          >
            <Download className="size-4" />
            {importMutation.isPending ? "Importing..." : "Trigger Import"}
          </Button>
        </div>

        {importResult && (
          <div className="flex flex-col gap-2 rounded-md border bg-muted/50 p-4 text-sm">
            <div className="flex items-center gap-2">
              <CheckCircle className="size-4 text-green-600" />
              <span>
                <strong>{importResult.imported}</strong> imported,{" "}
                <strong>{importResult.skipped}</strong> skipped
              </span>
            </div>
            {importResult.errors.length > 0 && (
              <div className="flex items-start gap-2 text-destructive">
                <AlertCircle className="mt-0.5 size-4 shrink-0" />
                <ul className="list-inside list-disc">
                  {importResult.errors.map((err, i) => (
                    <li key={i}>{err}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        )}

        {importMutation.error && (
          <p className="text-sm text-destructive">
            {importMutation.error instanceof Error
              ? importMutation.error.message
              : "Import failed"}
          </p>
        )}
      </section>

      {/* Configuration */}
      <section className="flex flex-col gap-4 rounded-lg border p-6">
        <div className="flex items-center gap-3">
          <FileText className="size-5 text-muted-foreground" />
          <h2 className="text-lg font-semibold">Configuration</h2>
        </div>
        <p className="text-sm text-muted-foreground">
          Application settings are managed via the{" "}
          <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">
            config.toml
          </code>{" "}
          file. Edit this file to configure:
        </p>
        <ul className="list-inside list-disc space-y-1 text-sm text-muted-foreground">
          <li>
            <strong>BBB connection</strong> &mdash; server URL and shared secret
          </li>
          <li>
            <strong>Storage</strong> &mdash; recording file storage directory
          </li>
          <li>
            <strong>Capture</strong> &mdash; ffmpeg path and output format
          </li>
          <li>
            <strong>Server</strong> &mdash; host, port, and database URL
          </li>
          <li>
            <strong>Auto-import</strong> &mdash; interval for background BBB
            import
          </li>
        </ul>
        <p className="text-xs text-muted-foreground">
          Changes to config.toml require a server restart to take effect.
        </p>
      </section>
    </div>
  );
}
