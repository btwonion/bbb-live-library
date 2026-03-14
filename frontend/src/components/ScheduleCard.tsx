import { Calendar, Clock, Edit2, Link, Repeat, Trash2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { Schedule } from "@/api/types";

interface ScheduleCardProps {
  schedule: Schedule;
  onEdit: (schedule: Schedule) => void;
  onDelete: (schedule: Schedule) => void;
}

function statusBadge(status: string) {
  switch (status) {
    case "recording":
      return (
        <Badge variant="destructive">
          <span className="mr-1 inline-block size-1.5 animate-pulse rounded-full bg-current" />
          Recording
        </Badge>
      );
    case "completed":
      return <Badge className="bg-green-600 text-white">Completed</Badge>;
    case "missed":
      return <Badge className="bg-yellow-500 text-white">Missed</Badge>;
    default:
      return <Badge>Pending</Badge>;
  }
}

function formatDateTime(iso: string) {
  return new Date(iso).toLocaleString(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  });
}

function truncate(str: string, maxLength: number) {
  return str.length > maxLength ? str.slice(0, maxLength) + "..." : str;
}

export function ScheduleCard({ schedule, onEdit, onDelete }: ScheduleCardProps) {
  const isRecording = schedule.status === "recording";

  return (
    <div className="flex items-start justify-between gap-4 rounded-lg border p-4">
      <div className="min-w-0 flex-1 space-y-2">
        <div className="flex items-center gap-2">
          <h3 className="truncate text-sm font-medium">{schedule.title}</h3>
          {statusBadge(schedule.status)}
          {!schedule.enabled && (
            <Badge variant="secondary">Disabled</Badge>
          )}
        </div>

        <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
          <span className="inline-flex items-center gap-1">
            <Link className="size-3" />
            {truncate(schedule.stream_url, 40)}
          </span>
          <span className="inline-flex items-center gap-1">
            <Calendar className="size-3" />
            {formatDateTime(schedule.start_time)}
          </span>
          {schedule.end_time && (
            <span className="inline-flex items-center gap-1">
              <Clock className="size-3" />
              Until {formatDateTime(schedule.end_time)}
            </span>
          )}
          {schedule.recurrence && (
            <span className="inline-flex items-center gap-1">
              <Repeat className="size-3" />
              {schedule.recurrence}
            </span>
          )}
        </div>
      </div>

      <div className="flex shrink-0 items-center gap-1">
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={() => onEdit(schedule)}
        >
          <Edit2 className="size-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={() => onDelete(schedule)}
          disabled={isRecording}
          title={isRecording ? "Cannot delete while recording" : "Delete schedule"}
        >
          <Trash2 className="size-4" />
        </Button>
      </div>
    </div>
  );
}
