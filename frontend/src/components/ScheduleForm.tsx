import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { HelpCircle } from "lucide-react";
import { createSchedule, updateSchedule } from "@/api/schedules";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { DateTimePicker } from "@/components/ui/date-time-picker";
import type { Schedule, CreateScheduleRequest, UpdateScheduleRequest } from "@/api/types";

interface ScheduleFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  schedule?: Schedule;
}

interface FormFieldsProps {
  schedule?: Schedule;
  onOpenChange: (open: boolean) => void;
}

function FormFields({ schedule, onOpenChange }: FormFieldsProps) {
  const isEdit = !!schedule;
  const queryClient = useQueryClient();

  const [title, setTitle] = useState(schedule?.title ?? "");
  const [streamUrl, setStreamUrl] = useState(schedule?.stream_url ?? "");
  const [startTime, setStartTime] = useState<Date | undefined>(
    schedule ? new Date(schedule.start_time) : undefined,
  );
  const [endTime, setEndTime] = useState<Date | undefined>(
    schedule?.end_time ? new Date(schedule.end_time) : undefined,
  );
  const [meetingId, setMeetingId] = useState(schedule?.meeting_id ?? "");
  const [recurrence, setRecurrence] = useState(schedule?.recurrence ?? "");

  const createMutation = useMutation({
    mutationFn: (data: CreateScheduleRequest) => createSchedule(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
      toast.success("Schedule created");
      onOpenChange(false);
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: UpdateScheduleRequest) => updateSchedule(schedule!.id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
      toast.success("Schedule updated");
      onOpenChange(false);
    },
  });

  const isPending = createMutation.isPending || updateMutation.isPending;
  const error = createMutation.error || updateMutation.error;

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();

    if (!startTime) {
      toast.error("Start time is required");
      return;
    }

    const payload = {
      title,
      stream_url: streamUrl,
      start_time: startTime.toISOString(),
      end_time: endTime ? endTime.toISOString() : undefined,
      meeting_id: meetingId || undefined,
      recurrence: recurrence || undefined,
    };

    if (isEdit) {
      updateMutation.mutate(payload as UpdateScheduleRequest);
    } else {
      createMutation.mutate(payload as CreateScheduleRequest);
    }
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-3">
      <div className="space-y-1">
        <label htmlFor="sf-title" className="text-xs font-medium">
          Title *
        </label>
        <Input
          id="sf-title"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          required
        />
      </div>

      <div className="space-y-1">
        <div className="flex items-center gap-1">
          <label htmlFor="sf-stream" className="text-xs font-medium">
            Stream URL *
          </label>
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger render={<span />}>
                <HelpCircle className="size-3.5 text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent side="right">
                <p>The RTMP stream URL from your BigBlueButton server.</p>
                <a
                  href="https://docs.bigbluebutton.org/administration/customize/#enable-live-streaming"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="underline"
                >
                  Learn more
                </a>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
        <Input
          id="sf-stream"
          value={streamUrl}
          onChange={(e) => setStreamUrl(e.target.value)}
          placeholder="rtmp://..."
          required
        />
      </div>

      <div className="grid grid-cols-2 gap-3">
        <div className="space-y-1">
          <label className="text-xs font-medium">Start Time *</label>
          <DateTimePicker value={startTime} onChange={setStartTime} />
        </div>
        <div className="space-y-1">
          <label className="text-xs font-medium">End Time</label>
          <DateTimePicker value={endTime} onChange={setEndTime} />
        </div>
      </div>

      <div className="space-y-1">
        <label htmlFor="sf-meeting" className="text-xs font-medium">
          Meeting ID
        </label>
        <Input
          id="sf-meeting"
          value={meetingId}
          onChange={(e) => setMeetingId(e.target.value)}
          placeholder="Optional BBB meeting ID"
        />
      </div>

      <div className="space-y-1">
        <label htmlFor="sf-recurrence" className="text-xs font-medium">
          Recurrence
        </label>
        <Input
          id="sf-recurrence"
          value={recurrence}
          onChange={(e) => setRecurrence(e.target.value)}
          placeholder="Cron expression (e.g. 0 9 * * MON)"
        />
      </div>

      {error && (
        <p className="text-xs text-destructive">
          {error instanceof Error ? error.message : "An error occurred"}
        </p>
      )}

      <DialogFooter>
        <Button
          type="button"
          variant="outline"
          onClick={() => onOpenChange(false)}
        >
          Cancel
        </Button>
        <Button type="submit" disabled={isPending}>
          {isPending
            ? isEdit
              ? "Saving..."
              : "Creating..."
            : isEdit
              ? "Save Changes"
              : "Create Schedule"}
        </Button>
      </DialogFooter>
    </form>
  );
}

export function ScheduleForm({ open, onOpenChange, schedule }: ScheduleFormProps) {
  const isEdit = !!schedule;
  const formKey = schedule?.id ?? "new";

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{isEdit ? "Edit Schedule" : "New Schedule"}</DialogTitle>
          <DialogDescription>
            {isEdit
              ? "Update the schedule details."
              : "Set up a new recording schedule."}
          </DialogDescription>
        </DialogHeader>
        <FormFields
          key={formKey}
          schedule={schedule}
          onOpenChange={onOpenChange}
        />
      </DialogContent>
    </Dialog>
  );
}
