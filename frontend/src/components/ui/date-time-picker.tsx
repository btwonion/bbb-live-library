import { useState } from "react";
import { format } from "date-fns";
import { CalendarIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import { Input } from "@/components/ui/input";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";

interface DateTimePickerProps {
  value: Date | undefined;
  onChange: (date: Date | undefined) => void;
  placeholder?: string;
}

export function DateTimePicker({
  value,
  onChange,
  placeholder = "Pick date & time",
}: DateTimePickerProps) {
  const [open, setOpen] = useState(false);

  function handleDateSelect(day: Date | undefined) {
    if (!day) {
      onChange(undefined);
      return;
    }
    const next = new Date(day);
    if (value) {
      next.setHours(value.getHours(), value.getMinutes());
    }
    onChange(next);
  }

  function handleTimeChange(field: "hours" | "minutes", raw: string) {
    const num = parseInt(raw, 10);
    if (isNaN(num)) return;
    const base = value ? new Date(value) : new Date();
    if (field === "hours" && num >= 0 && num <= 23) {
      base.setHours(num);
    } else if (field === "minutes" && num >= 0 && num <= 59) {
      base.setMinutes(num);
    } else {
      return;
    }
    onChange(base);
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger
        render={
          <Button
            variant="outline"
            className={cn(
              "h-8 w-full justify-start text-left text-sm font-normal",
              !value && "text-muted-foreground",
            )}
          />
        }
      >
        <CalendarIcon className="mr-2 size-4" />
        {value ? format(value, "MMM d, yyyy HH:mm") : placeholder}
      </PopoverTrigger>
      <PopoverContent align="start" className="z-[60] w-auto p-0">
        <Calendar
          mode="single"
          selected={value}
          onSelect={handleDateSelect}
        />
        <Separator />
        <div className="flex items-center gap-2 px-3 py-2">
          <Input
            type="number"
            min={0}
            max={23}
            value={value ? String(value.getHours()).padStart(2, "0") : ""}
            onChange={(e) => handleTimeChange("hours", e.target.value)}
            className="h-8 w-16 text-center"
            placeholder="HH"
          />
          <span className="text-sm text-muted-foreground">:</span>
          <Input
            type="number"
            min={0}
            max={59}
            value={value ? String(value.getMinutes()).padStart(2, "0") : ""}
            onChange={(e) => handleTimeChange("minutes", e.target.value)}
            className="h-8 w-16 text-center"
            placeholder="MM"
          />
        </div>
      </PopoverContent>
    </Popover>
  );
}
