import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Clock, Download, FileText, CheckCircle } from "lucide-react";
import { toast } from "sonner";
import { importPublicBbb } from "@/api/import";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { useTimezone } from "@/hooks/useTimezone";

export default function SettingsPage() {
  useDocumentTitle("Settings");
  const timezone = useTimezone();

  const [bbbUrl, setBbbUrl] = useState("");
  const [title, setTitle] = useState("");
  const [importedTitle, setImportedTitle] = useState<string | null>(null);

  const importMutation = useMutation({
    mutationFn: importPublicBbb,
    onSuccess: (data) => {
      setImportedTitle(data.title);
      toast.success(`Imported: ${data.title}`);
      setBbbUrl("");
      setTitle("");
    },
  });

  return (
    <div className="flex flex-col gap-8">
      <h1 className="text-2xl font-bold">Settings</h1>

      {/* Public BBB Import */}
      <section className="flex flex-col gap-4 rounded-lg border p-6">
        <div className="flex items-center gap-3">
          <Download className="size-5 text-muted-foreground" />
          <h2 className="text-lg font-semibold">Public BBB Import</h2>
        </div>
        <p className="text-sm text-muted-foreground">
          Import a recording from a public BigBlueButton server. Paste a full
          playback URL.
        </p>

        <div className="flex flex-col gap-3">
          <div className="flex flex-col gap-1.5">
            <label htmlFor="bbb-url" className="text-sm font-medium">BBB Recording URL</label>
            <Input
              id="bbb-url"
              placeholder="https://bbb.example.com/playback/presentation/2.3/record-id"
              value={bbbUrl}
              onChange={(e) => setBbbUrl(e.target.value)}
            />
          </div>

          <div className="flex flex-col gap-1.5">
            <label htmlFor="title" className="text-sm font-medium">Title (optional)</label>
            <Input
              id="title"
              placeholder="Override the recording title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
            />
          </div>

          <div className="flex items-center gap-4">
            <Button
              onClick={() => {
                setImportedTitle(null);
                importMutation.mutate({
                  url: bbbUrl,
                  title: title || undefined,
                });
              }}
              disabled={importMutation.isPending || !bbbUrl}
            >
              <Download className="size-4" />
              {importMutation.isPending ? "Importing..." : "Import Recording"}
            </Button>
          </div>
        </div>

        {importedTitle && (
          <div className="flex items-center gap-2 rounded-md border bg-muted/50 p-4 text-sm">
            <CheckCircle className="size-4 text-green-600" />
            <span>
              Successfully imported: <strong>{importedTitle}</strong>
            </span>
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

      {/* Timezone */}
      <section className="flex flex-col gap-4 rounded-lg border p-6">
        <div className="flex items-center gap-3">
          <Clock className="size-5 text-muted-foreground" />
          <h2 className="text-lg font-semibold">Timezone</h2>
        </div>
        <p className="text-sm text-muted-foreground">
          Schedule times are displayed and interpreted in this timezone.
          Configured via{" "}
          <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">
            timezone
          </code>{" "}
          in{" "}
          <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">
            config.toml
          </code>
          .
        </p>
        <div className="flex items-center gap-2 rounded-md border bg-muted/50 p-4 text-sm font-medium">
          <Clock className="size-4 text-muted-foreground" />
          {timezone}
        </div>
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
            <strong>Storage</strong> &mdash; recording file storage directory
          </li>
          <li>
            <strong>Capture</strong> &mdash; ffmpeg path and output format
          </li>
          <li>
            <strong>Server</strong> &mdash; host, port, and database URL
          </li>
        </ul>
        <p className="text-xs text-muted-foreground">
          Changes to config.toml require a server restart to take effect.
        </p>
      </section>
    </div>
  );
}
