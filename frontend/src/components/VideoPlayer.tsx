import { Video } from "lucide-react";
import { getStreamUrl, getThumbnailUrl } from "@/api/recordings";

interface VideoPlayerProps {
  recordingId: string;
  hasThumbnail: boolean;
}

export function VideoPlayer({ recordingId, hasThumbnail }: VideoPlayerProps) {
  return (
    <div className="relative aspect-video overflow-hidden rounded-lg bg-black">
      <video
        className="size-full"
        controls
        src={getStreamUrl(recordingId)}
        poster={hasThumbnail ? getThumbnailUrl(recordingId) : undefined}
      >
        <track kind="captions" />
      </video>
      {!hasThumbnail && (
        <div className="pointer-events-none absolute inset-0 flex items-center justify-center text-muted-foreground">
          <Video className="size-16 opacity-20" />
        </div>
      )}
    </div>
  );
}
