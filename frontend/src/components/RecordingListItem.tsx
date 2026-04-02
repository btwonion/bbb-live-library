import { Link } from "react-router-dom";
import { Video } from "lucide-react";
import { formatDuration } from "@/lib/formatDuration";
import { getThumbnailUrl } from "@/api/recordings";
import type { Recording } from "@/api/types";

interface RecordingListItemProps {
  recording: Recording;
}

export function RecordingListItem({ recording }: RecordingListItemProps) {
  const date = new Date(recording.created_at).toLocaleDateString();

  return (
    <Link
      to={`/recordings/${recording.id}`}
      className="flex items-center gap-4 rounded-lg border bg-card p-3 transition-colors hover:bg-accent/50"
    >
      <div className="relative h-20 w-40 shrink-0 overflow-hidden rounded-md bg-muted">
        {recording.thumbnail_path ? (
          <img
            src={getThumbnailUrl(recording.id)}
            alt={recording.title}
            className="size-full object-cover"
          />
        ) : (
          <div className="flex size-full items-center justify-center text-muted-foreground">
            <Video className="size-8" />
          </div>
        )}
      </div>
      <div className="min-w-0 flex-1">
        <h3 className="truncate text-sm font-medium">{recording.title}</h3>
        {recording.description && (
          <p className="mt-0.5 line-clamp-1 text-xs text-muted-foreground">
            {recording.description}
          </p>
        )}
      </div>
      <div className="flex shrink-0 items-center gap-3 text-xs text-muted-foreground">
        {recording.duration_seconds != null && (
          <span className="font-mono">
            {formatDuration(recording.duration_seconds)}
          </span>
        )}
        <span className="whitespace-nowrap">{date}</span>
      </div>
    </Link>
  );
}
