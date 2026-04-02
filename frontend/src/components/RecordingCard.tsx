import { Link } from "react-router-dom";
import { Video } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { formatDuration } from "@/lib/formatDuration";
import { getThumbnailUrl } from "@/api/recordings";
import type { Recording } from "@/api/types";

interface RecordingCardProps {
  recording: Recording;
}

export function RecordingCard({ recording }: RecordingCardProps) {
  const date = new Date(recording.created_at).toLocaleDateString();

  return (
    <Link
      to={`/recordings/${recording.id}`}
      className="group block overflow-hidden rounded-lg border bg-card transition-colors hover:bg-accent/50"
    >
      <div className="relative aspect-video bg-muted">
        {recording.thumbnail_path ? (
          <img
            src={getThumbnailUrl(recording.id)}
            alt={recording.title}
            className="size-full object-cover"
          />
        ) : (
          <div className="flex size-full items-center justify-center text-muted-foreground">
            <Video className="size-12" />
          </div>
        )}
        {recording.duration_seconds != null && (
          <Badge
            variant="secondary"
            className="absolute bottom-2 right-2 bg-black/70 text-white"
          >
            {formatDuration(recording.duration_seconds)}
          </Badge>
        )}
      </div>
      <div className="p-3">
        <h3 className="line-clamp-2 text-sm font-medium leading-snug">
          {recording.title}
        </h3>
        <div className="mt-1.5 text-xs text-muted-foreground">
          <span>{date}</span>
        </div>
      </div>
    </Link>
  );
}
