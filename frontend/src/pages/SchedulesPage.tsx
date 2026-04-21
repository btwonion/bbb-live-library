import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, CalendarClock } from "lucide-react";
import { toast } from "sonner";
import { listSchedules, deleteSchedule } from "@/api/schedules";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { ScheduleCard } from "@/components/ScheduleCard";
import { ScheduleForm } from "@/components/ScheduleForm";
import { Pagination } from "@/components/Pagination";
import type { Schedule } from "@/api/types";

const PER_PAGE = 10;

export default function SchedulesPage() {
  useDocumentTitle("Schedules");

  const queryClient = useQueryClient();
  const [searchParams, setSearchParams] = useSearchParams();
  const activePage = Number(searchParams.get("page")) || 1;
  const pastPage = Number(searchParams.get("past_page")) || 1;

  const [formOpen, setFormOpen] = useState(false);
  const [editingSchedule, setEditingSchedule] = useState<Schedule | undefined>();
  const [deletingSchedule, setDeletingSchedule] = useState<Schedule | null>(null);

  const { data: activeData, isLoading: activeLoading } = useQuery({
    queryKey: ["schedules", { page: activePage, per_page: PER_PAGE, filter: "active" }],
    queryFn: () => listSchedules({ page: activePage, per_page: PER_PAGE, filter: "active" }),
    refetchInterval: 30000,
  });

  const { data: pastData, isLoading: pastLoading } = useQuery({
    queryKey: ["schedules", { page: pastPage, per_page: PER_PAGE, filter: "past" }],
    queryFn: () => listSchedules({ page: pastPage, per_page: PER_PAGE, filter: "past" }),
    refetchInterval: 30000,
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteSchedule(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
      toast.success("Schedule deleted");
      setDeletingSchedule(null);
    },
  });

  const activeTotalPages = activeData ? Math.ceil(activeData.total / PER_PAGE) : 0;
  const pastTotalPages = pastData ? Math.ceil(pastData.total / PER_PAGE) : 0;

  function handleEdit(schedule: Schedule) {
    setEditingSchedule(schedule);
    setFormOpen(true);
  }

  function handleCreate() {
    setEditingSchedule(undefined);
    setFormOpen(true);
  }

  function handleActivePageChange(p: number) {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      if (p > 1) {
        next.set("page", String(p));
      } else {
        next.delete("page");
      }
      return next;
    });
  }

  function handlePastPageChange(p: number) {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      if (p > 1) {
        next.set("past_page", String(p));
      } else {
        next.delete("past_page");
      }
      return next;
    });
  }

  const isEmpty = !activeLoading && !pastLoading &&
    activeData?.data.length === 0 && pastData?.data.length === 0;

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Schedules</h1>
        <Button onClick={handleCreate}>
          <Plus className="size-4" />
          New Schedule
        </Button>
      </div>

      {(activeLoading || pastLoading) && (
        <div className="space-y-3">
          {Array.from({ length: 3 }).map((_, i) => (
            <Skeleton key={i} className="h-20 w-full rounded-lg" />
          ))}
        </div>
      )}

      {isEmpty && (
        <div className="flex flex-col items-center gap-3 rounded-lg border border-dashed p-12 text-center">
          <CalendarClock className="size-10 text-muted-foreground" />
          <div>
            <p className="font-medium">No schedules yet</p>
            <p className="text-sm text-muted-foreground">
              Create your first recording schedule to get started.
            </p>
          </div>
          <Button onClick={handleCreate}>
            <Plus className="size-4" />
            New Schedule
          </Button>
        </div>
      )}

      {!activeLoading && activeData && activeData.data.length > 0 && (
        <div className="space-y-3">
          <h2 className="text-lg font-semibold">Active Schedules</h2>
          {activeData.data.map((schedule) => (
            <ScheduleCard
              key={schedule.id}
              schedule={schedule}
              onEdit={handleEdit}
              onDelete={setDeletingSchedule}
            />
          ))}
          {activeTotalPages > 1 && (
            <Pagination
              page={activePage}
              totalPages={activeTotalPages}
              onPageChange={handleActivePageChange}
            />
          )}
        </div>
      )}

      {!pastLoading && pastData && pastData.data.length > 0 && (
        <div className="space-y-3">
          <h2 className="text-lg font-semibold text-muted-foreground">Past Schedules</h2>
          {pastData.data.map((schedule) => (
            <ScheduleCard
              key={schedule.id}
              schedule={schedule}
              onEdit={handleEdit}
              onDelete={setDeletingSchedule}
            />
          ))}
          {pastTotalPages > 1 && (
            <Pagination
              page={pastPage}
              totalPages={pastTotalPages}
              onPageChange={handlePastPageChange}
            />
          )}
        </div>
      )}

      <ScheduleForm
        open={formOpen}
        onOpenChange={setFormOpen}
        schedule={editingSchedule}
      />

      <AlertDialog
        open={!!deletingSchedule}
        onOpenChange={(open) => {
          if (!open) setDeletingSchedule(null);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete schedule?</AlertDialogTitle>
            <AlertDialogDescription>
              This will permanently delete &ldquo;{deletingSchedule?.title}&rdquo;.
              This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={() => {
                if (deletingSchedule) deleteMutation.mutate(deletingSchedule.id);
              }}
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending ? "Deleting..." : "Delete"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
