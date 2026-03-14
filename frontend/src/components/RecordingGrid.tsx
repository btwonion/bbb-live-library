import { Skeleton } from "@/components/ui/skeleton";
import { RecordingCard } from "@/components/RecordingCard";
import { RecordingListItem } from "@/components/RecordingListItem";
import { Video } from "lucide-react";
import type { Recording } from "@/api/types";

interface RecordingGridProps {
  recordings: Recording[] | undefined;
  isLoading: boolean;
  viewMode: "grid" | "list";
  hasFilters: boolean;
}

export function RecordingGrid({
  recordings,
  isLoading,
  viewMode,
  hasFilters,
}: RecordingGridProps) {
  if (isLoading) {
    return viewMode === "grid" ? <GridSkeletons /> : <ListSkeletons />;
  }

  if (!recordings || recordings.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center gap-3 py-20 text-muted-foreground">
        <Video className="size-12" />
        <p className="text-lg font-medium">
          {hasFilters ? "No recordings match your filters" : "No recordings yet"}
        </p>
        {hasFilters && (
          <p className="text-sm">Try adjusting your search or filters.</p>
        )}
      </div>
    );
  }

  if (viewMode === "list") {
    return (
      <div className="flex flex-col gap-2">
        {recordings.map((r) => (
          <RecordingListItem key={r.id} recording={r} />
        ))}
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
      {recordings.map((r) => (
        <RecordingCard key={r.id} recording={r} />
      ))}
    </div>
  );
}

function GridSkeletons() {
  return (
    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
      {Array.from({ length: 12 }, (_, i) => (
        <div key={i} className="overflow-hidden rounded-lg border">
          <Skeleton className="aspect-video w-full rounded-none" />
          <div className="p-3">
            <Skeleton className="mb-2 h-4 w-3/4" />
            <Skeleton className="h-3 w-1/2" />
          </div>
        </div>
      ))}
    </div>
  );
}

function ListSkeletons() {
  return (
    <div className="flex flex-col gap-2">
      {Array.from({ length: 8 }, (_, i) => (
        <div key={i} className="flex items-center gap-4 rounded-lg border p-3">
          <Skeleton className="h-20 w-40 shrink-0 rounded-md" />
          <div className="flex-1">
            <Skeleton className="mb-2 h-4 w-1/2" />
            <Skeleton className="h-3 w-1/3" />
          </div>
        </div>
      ))}
    </div>
  );
}
