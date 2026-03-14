import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Check, Pencil, X } from "lucide-react";
import { toast } from "sonner";
import { updateRecording } from "@/api/recordings";
import type { RecordingDetail } from "@/api/types";
import { formatDuration } from "@/lib/formatDuration";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";

interface MetadataPanelProps {
  recording: RecordingDetail;
}

function sourceLabel(source: string): string {
  return source === "live_capture" ? "Live" : "Import";
}

export function MetadataPanel({ recording }: MetadataPanelProps) {
  const queryClient = useQueryClient();
  const [editingTitle, setEditingTitle] = useState(false);
  const [editingDescription, setEditingDescription] = useState(false);
  const [title, setTitle] = useState(recording.title);
  const [description, setDescription] = useState(recording.description ?? "");

  const mutation = useMutation({
    mutationFn: (data: { title?: string; description?: string }) =>
      updateRecording(recording.id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["recording", recording.id] });
      toast.success("Recording updated");
    },
  });

  function saveTitle() {
    if (title.trim() && title !== recording.title) {
      mutation.mutate({ title: title.trim() });
    }
    setEditingTitle(false);
  }

  function cancelTitle() {
    setTitle(recording.title);
    setEditingTitle(false);
  }

  function saveDescription() {
    const trimmed = description.trim();
    if (trimmed !== (recording.description ?? "")) {
      mutation.mutate({ description: trimmed });
    }
    setEditingDescription(false);
  }

  function cancelDescription() {
    setDescription(recording.description ?? "");
    setEditingDescription(false);
  }

  const date = new Date(recording.created_at).toLocaleDateString(undefined, {
    year: "numeric",
    month: "long",
    day: "numeric",
  });

  return (
    <div className="space-y-4">
      {/* Title */}
      <div className="group">
        {editingTitle ? (
          <div className="flex items-center gap-2">
            <Input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") saveTitle();
                if (e.key === "Escape") cancelTitle();
              }}
              className="text-lg font-semibold"
              autoFocus
            />
            <Button size="icon-xs" variant="ghost" onClick={saveTitle}>
              <Check />
            </Button>
            <Button size="icon-xs" variant="ghost" onClick={cancelTitle}>
              <X />
            </Button>
          </div>
        ) : (
          <div className="flex items-center gap-2">
            <h1 className="text-xl font-semibold">{recording.title}</h1>
            <Button
              size="icon-xs"
              variant="ghost"
              className="opacity-0 group-hover:opacity-100"
              onClick={() => setEditingTitle(true)}
            >
              <Pencil />
            </Button>
          </div>
        )}
      </div>

      {/* Description */}
      <div className="group">
        {editingDescription ? (
          <div className="space-y-2">
            <Textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Escape") cancelDescription();
              }}
              placeholder="Add a description..."
              autoFocus
            />
            <div className="flex gap-2">
              <Button size="sm" onClick={saveDescription}>
                Save
              </Button>
              <Button size="sm" variant="outline" onClick={cancelDescription}>
                Cancel
              </Button>
            </div>
          </div>
        ) : (
          <div className="flex items-start gap-2">
            <p
              className="cursor-pointer text-sm text-muted-foreground"
              onClick={() => setEditingDescription(true)}
            >
              {recording.description || "No description. Click to add one."}
            </p>
            <Button
              size="icon-xs"
              variant="ghost"
              className="mt-0.5 shrink-0 opacity-0 group-hover:opacity-100"
              onClick={() => setEditingDescription(true)}
            >
              <Pencil />
            </Button>
          </div>
        )}
      </div>

      {/* Metadata grid */}
      <div className="flex flex-wrap items-center gap-3 text-sm text-muted-foreground">
        <Badge variant="outline">{sourceLabel(recording.source)}</Badge>
        {recording.duration_seconds != null && (
          <span>{formatDuration(recording.duration_seconds)}</span>
        )}
        <span>{recording.format.toUpperCase()}</span>
        <span>{date}</span>
      </div>
    </div>
  );
}
