import type { LucideIcon } from "lucide-react";

interface StatsCardProps {
  icon: LucideIcon;
  label: string;
  value: string | number;
}

export function StatsCard({ icon: Icon, label, value }: StatsCardProps) {
  return (
    <div className="flex items-center gap-4 rounded-lg border bg-card p-4">
      <div className="flex size-10 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
        <Icon className="size-5" />
      </div>
      <div className="min-w-0">
        <p className="text-sm text-muted-foreground">{label}</p>
        <p className="truncate text-2xl font-bold">{value}</p>
      </div>
    </div>
  );
}
