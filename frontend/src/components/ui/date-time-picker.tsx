import { useEffect, useRef, useState } from "react";
import { format } from "date-fns";
import { CalendarIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";

interface DateTimePickerProps {
  value: Date | undefined;
  onChange: (date: Date | undefined) => void;
  placeholder?: string;
}

const HOURS = Array.from({ length: 24 }, (_, i) => i);
const MINUTES = Array.from({ length: 60 }, (_, i) => i);

function TimeColumn({
  values,
  selected,
  onSelect,
}: {
  values: number[];
  selected: number;
  onSelect: (v: number) => void;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const selectedRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (selectedRef.current) {
      selectedRef.current.scrollIntoView({
        block: "center",
        behavior: "instant",
      });
    }
  }, [selected]);

  return (
    <div
      ref={containerRef}
      className="w-16 overflow-y-auto scrollbar-thin"
    >
      <div>
        {values.map((v) => (
          <button
            key={v}
            ref={v === selected ? selectedRef : undefined}
            type="button"
            onClick={() => onSelect(v)}
            className={cn(
              "flex h-8 w-full items-center justify-center rounded-md text-sm transition-colors",
              "hover:bg-muted",
              v === selected &&
                "bg-primary text-primary-foreground font-medium hover:bg-primary/90",
            )}
          >
            {String(v).padStart(2, "0")}
          </button>
        ))}
      </div>
    </div>
  );
}

export function DateTimePicker({
  value,
  onChange,
  placeholder = "Pick date & time",
}: DateTimePickerProps) {
  const [open, setOpen] = useState(false);
  const [fallbackDate, setFallbackDate] = useState<Date>(() => new Date());

  const displayDate = value ?? fallbackDate;

  function handleOpenChange(nextOpen: boolean) {
    if (nextOpen && !value) {
      setFallbackDate(new Date());
    }
    setOpen(nextOpen);
  }

  function handleDateSelect(day: Date | undefined) {
    if (!day) {
      onChange(undefined);
      return;
    }
    const next = new Date(day);
    const timeSource = value ?? displayDate;
    next.setHours(timeSource.getHours(), timeSource.getMinutes());
    onChange(next);
  }

  function handleTimeChange(field: "hours" | "minutes", num: number) {
    const base = value ? new Date(value) : new Date(displayDate);
    if (field === "hours") {
      base.setHours(num);
    } else {
      base.setMinutes(num);
    }
    onChange(base);
  }

  const displayHours = (value ?? displayDate).getHours();
  const displayMinutes = (value ?? displayDate).getMinutes();

  return (
    <Popover open={open} onOpenChange={handleOpenChange}>
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
        <div className="flex">
          <Calendar
            mode="single"
            selected={value}
            defaultMonth={displayDate}
            onSelect={handleDateSelect}
          />
          <Separator orientation="vertical" />
          <div className="flex max-h-[300px]">
            <TimeColumn
              values={HOURS}
              selected={displayHours}
              onSelect={(h) => handleTimeChange("hours", h)}
            />
            <Separator orientation="vertical" />
            <TimeColumn
              values={MINUTES}
              selected={displayMinutes}
              onSelect={(m) => handleTimeChange("minutes", m)}
            />
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}
