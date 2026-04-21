import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { ArrowRight, Edit2, MoreHorizontal, Trash2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { RecordingCard } from "@/components/RecordingCard";
import { listRecordings } from "@/api/recordings";
import type { Category } from "@/api/types";

interface CategorySectionProps {
  category: Category;
  recordingCount: number;
  onEdit: (category: Category) => void;
  onDelete: (category: Category) => void;
}

export function CategorySection({
  category,
  recordingCount,
  onEdit,
  onDelete,
}: CategorySectionProps) {
  const { data, isLoading } = useQuery({
    queryKey: ["recordings", { category_id: category.id, per_page: 4 }],
    queryFn: () => listRecordings({ category_id: category.id, per_page: 4 }),
    staleTime: 0,
  });

  const recordings = data?.data ?? [];

  return (
    <section className="space-y-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h2 className="text-lg font-semibold">{category.name}</h2>
          <Badge variant="secondary">
            {recordingCount} {recordingCount === 1 ? "recording" : "recordings"}
          </Badge>
        </div>
        <div className="flex items-center gap-2">
          {recordingCount > 0 && (
            <Link
              to={`/recordings?category=${category.id}`}
              className="inline-flex items-center gap-1 rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
            >
              View all
              <ArrowRight className="size-4" />
            </Link>
          )}
          <DropdownMenu>
            <DropdownMenuTrigger className="inline-flex size-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground">
              <MoreHorizontal className="size-4" />
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => onEdit(category)}>
                <Edit2 className="mr-2 size-4" />
                Edit
              </DropdownMenuItem>
              <DropdownMenuItem
                className="text-destructive"
                onClick={() => onDelete(category)}
              >
                <Trash2 className="mr-2 size-4" />
                Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>

      {category.description && (
        <p className="text-sm text-muted-foreground">{category.description}</p>
      )}

      {isLoading ? (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <Skeleton key={i} className="aspect-video rounded-lg" />
          ))}
        </div>
      ) : recordings.length > 0 ? (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {recordings.map((recording) => (
            <RecordingCard key={recording.id} recording={recording} />
          ))}
        </div>
      ) : (
        <div className="rounded-lg border border-dashed p-6 text-center text-sm text-muted-foreground">
          No recordings in this category yet.
        </div>
      )}
    </section>
  );
}
