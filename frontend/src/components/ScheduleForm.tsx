import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { HelpCircle } from "lucide-react";
import { listCategories } from "@/api/categories";
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
  const [recordingType, setRecordingType] = useState<"rtmp" | "room">(
    schedule?.stream_url ? "rtmp" : "room",
  );
  const [streamUrl, setStreamUrl] = useState(schedule?.stream_url ?? "");
  const [roomUrl, setRoomUrl] = useState(schedule?.room_url ?? "");
  const [botName, setBotName] = useState(schedule?.bot_name ?? "");
  const [startTime, setStartTime] = useState<Date | undefined>(
    schedule ? new Date(schedule.start_time) : undefined,
  );
  const [endTime, setEndTime] = useState<Date | undefined>(
    schedule?.end_time ? new Date(schedule.end_time) : undefined,
  );
  const [recurrence, setRecurrence] = useState(schedule?.recurrence ?? "");
  const [startOffset, setStartOffset] = useState(schedule?.start_offset_secs ?? 30);
  const [endOffset, setEndOffset] = useState(schedule?.end_offset_secs ?? 30);
  const [categoryId, setCategoryId] = useState(schedule?.category_id ?? "");

  const { data: categories } = useQuery({
    queryKey: ["categories"],
    queryFn: listCategories,
  });

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
      stream_url: recordingType === "rtmp" ? streamUrl : "",
      room_url: recordingType === "room" ? roomUrl : "",
      bot_name: recordingType === "room" ? botName || undefined : undefined,
      start_time: startTime.toISOString(),
      end_time: endTime ? endTime.toISOString() : undefined,
      recurrence: recurrence || undefined,
      start_offset_secs: startOffset,
      end_offset_secs: endOffset,
      category_id: categoryId || (isEdit ? null : undefined),
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
        <label className="text-xs font-medium">Recording Source *</label>
        <div className="flex gap-1">
          <Button
            type="button"
            size="sm"
            variant={recordingType === "room" ? "default" : "outline"}
            className="flex-1"
            onClick={() => setRecordingType("room")}
          >
            BBB Room
          </Button>
          <Button
            type="button"
            size="sm"
            variant={recordingType === "rtmp" ? "default" : "outline"}
            className="flex-1"
            onClick={() => setRecordingType("rtmp")}
          >
            RTMP Stream
          </Button>
        </div>
      </div>

      {recordingType === "rtmp" ? (
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
      ) : (
        <>
          <div className="space-y-1">
            <label htmlFor="sf-room-url" className="text-xs font-medium">
              Room URL *
            </label>
            <Input
              id="sf-room-url"
              value={roomUrl}
              onChange={(e) => setRoomUrl(e.target.value)}
              placeholder="https://bbb.example.com/rooms/.../join"
              required
            />
          </div>
          <div className="space-y-1">
            <label htmlFor="sf-bot-name" className="text-xs font-medium">
              Bot Name
            </label>
            <Input
              id="sf-bot-name"
              value={botName}
              onChange={(e) => setBotName(e.target.value)}
              placeholder="Recorder"
            />
          </div>
        </>
      )}

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

      <div className="grid grid-cols-2 gap-3">
        <div className="space-y-1">
          <label htmlFor="sf-start-offset" className="text-xs font-medium">
            Start Offset (seconds)
          </label>
          <Input
            id="sf-start-offset"
            type="number"
            min={0}
            value={startOffset}
            onChange={(e) => setStartOffset(Number(e.target.value))}
          />
        </div>
        <div className="space-y-1">
          <label htmlFor="sf-end-offset" className="text-xs font-medium">
            End Offset (seconds)
          </label>
          <Input
            id="sf-end-offset"
            type="number"
            min={0}
            value={endOffset}
            onChange={(e) => setEndOffset(Number(e.target.value))}
          />
        </div>
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

      <div className="space-y-1">
        <label htmlFor="sf-category" className="text-xs font-medium">
          Auto-assign Category
        </label>
        <select
          id="sf-category"
          value={categoryId}
          onChange={(e) => setCategoryId(e.target.value)}
          className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
        >
          <option value="">None</option>
          {categories?.map((cat) => (
            <option key={cat.id} value={cat.id}>
              {cat.name}
            </option>
          ))}
        </select>
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
