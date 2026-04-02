import { useState } from "react";
import { useParams, useNavigate, Link } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { getRecording, deleteRecording } from "@/api/recordings";
import { VideoPlayer } from "@/components/VideoPlayer";
import { MetadataPanel } from "@/components/MetadataPanel";
import { CategoryEditor } from "@/components/CategoryEditor";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";

export default function RecordingDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  const {
    data: recording,
    isLoading,
    error,
  } = useQuery({
    queryKey: ["recording", id],
    queryFn: () => getRecording(id!),
    enabled: !!id,
  });

  useDocumentTitle(recording?.title ?? "Recording");

  const deleteMutation = useMutation({
    mutationFn: () => deleteRecording(id!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["recordings"] });
      toast.success("Recording deleted");
      navigate("/recordings");
    },
  });

  if (isLoading) {
    return (
      <div className="mx-auto max-w-4xl space-y-6 p-6">
        <Skeleton className="h-8 w-32" />
        <Skeleton className="aspect-video w-full rounded-lg" />
        <Skeleton className="h-8 w-64" />
        <Skeleton className="h-4 w-96" />
        <Skeleton className="h-20 w-full" />
      </div>
    );
  }

  if (error || !recording) {
    return (
      <div className="mx-auto max-w-4xl p-6">
        <Link
          to="/recordings"
          className="mb-4 inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="size-4" />
          Back to recordings
        </Link>
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-8 text-center">
          <p className="text-lg font-medium">Recording not found</p>
          <p className="mt-1 text-sm text-muted-foreground">
            This recording may have been deleted.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-4xl space-y-6 p-6">
      {/* Back link */}
      <Link
        to="/recordings"
        className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" />
        Back to recordings
      </Link>

      {/* Video player */}
      <VideoPlayer
        recordingId={recording.id}
        hasThumbnail={!!recording.thumbnail_path}
      />

      {/* Metadata */}
      <MetadataPanel recording={recording} />

      {/* Categories */}
      <CategoryEditor recording={recording} />

      {/* Delete */}
      <div className="border-t pt-6">
        <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
          <AlertDialogTrigger
            render={
              <Button variant="destructive">
                <Trash2 />
                Delete Recording
              </Button>
            }
          />
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>Delete recording?</AlertDialogTitle>
              <AlertDialogDescription>
                This will permanently delete &ldquo;{recording.title}&rdquo; and
                its associated files. This action cannot be undone.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>Cancel</AlertDialogCancel>
              <AlertDialogAction
                variant="destructive"
                onClick={() => deleteMutation.mutate()}
                disabled={deleteMutation.isPending}
              >
                {deleteMutation.isPending ? "Deleting..." : "Delete"}
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </div>
    </div>
  );
}
