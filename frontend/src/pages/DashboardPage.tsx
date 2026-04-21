import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import {
  ArrowRight,
  Film,
  HardDrive,
} from "lucide-react";
import { getStats } from "@/api/stats";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { listRecordings } from "@/api/recordings";
import { listSchedules } from "@/api/schedules";
import { formatDuration } from "@/lib/formatDuration";
import { buttonVariants } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { StatsCard } from "@/components/StatsCard";
import { RecordingCard } from "@/components/RecordingCard";
import { ScheduleCard } from "@/components/ScheduleCard";


export default function DashboardPage() {
  useDocumentTitle("Dashboard");

  const statsQuery = useQuery({
    queryKey: ["stats"],
    queryFn: getStats,
  });

  const recordingsQuery = useQuery({
    queryKey: ["recordings", { page: 1, per_page: 6 }],
    queryFn: () => listRecordings({ page: 1, per_page: 6 }),
  });

  const schedulesQuery = useQuery({
    queryKey: ["schedules", { page: 1, per_page: 5 }],
    queryFn: () => listSchedules({ page: 1, per_page: 5 }),
  });

  const stats = statsQuery.data;

  return (
    <div className="flex flex-col gap-8">
      <h1 className="text-2xl font-bold">Dashboard</h1>

      {/* Stats row */}
      {statsQuery.isLoading ? (
        <div className="grid gap-4 sm:grid-cols-2">
          {Array.from({ length: 2 }).map((_, i) => (
            <Skeleton key={i} className="h-20 rounded-lg" />
          ))}
        </div>
      ) : stats ? (
        <div className="grid gap-4 sm:grid-cols-2">
          <StatsCard
            icon={Film}
            label="Total Recordings"
            value={stats.recording_count}
          />
          <StatsCard
            icon={HardDrive}
            label="Total Duration"
            value={formatDuration(stats.total_duration_seconds)}
          />
        </div>
      ) : null}

      {/* Recent recordings */}
      <section className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold">Recent Recordings</h2>
          <Link
            to="/recordings"
            className={buttonVariants({ variant: "ghost", size: "sm" })}
          >
            View all <ArrowRight className="ml-1 size-4" />
          </Link>
        </div>

        {recordingsQuery.isLoading && (
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {Array.from({ length: 3 }).map((_, i) => (
              <Skeleton key={i} className="aspect-video rounded-lg" />
            ))}
          </div>
        )}

        {!recordingsQuery.isLoading &&
          recordingsQuery.data &&
          recordingsQuery.data.data.length === 0 && (
            <p className="text-sm text-muted-foreground">
              No recordings yet.
            </p>
          )}

        {!recordingsQuery.isLoading &&
          recordingsQuery.data &&
          recordingsQuery.data.data.length > 0 && (
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {recordingsQuery.data.data.map((recording) => (
                <RecordingCard key={recording.id} recording={recording} />
              ))}
            </div>
          )}
      </section>

      {/* Upcoming schedules */}
      <section className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold">Upcoming Schedules</h2>
          <Link
            to="/schedules"
            className={buttonVariants({ variant: "ghost", size: "sm" })}
          >
            View all <ArrowRight className="ml-1 size-4" />
          </Link>
        </div>

        {schedulesQuery.isLoading && (
          <div className="space-y-3">
            {Array.from({ length: 3 }).map((_, i) => (
              <Skeleton key={i} className="h-20 w-full rounded-lg" />
            ))}
          </div>
        )}

        {!schedulesQuery.isLoading &&
          schedulesQuery.data &&
          schedulesQuery.data.data.length === 0 && (
            <p className="text-sm text-muted-foreground">
              No upcoming schedules.
            </p>
          )}

        {!schedulesQuery.isLoading &&
          schedulesQuery.data &&
          schedulesQuery.data.data.length > 0 && (
            <div className="space-y-3">
              {schedulesQuery.data.data.map((schedule) => (
                <ScheduleCard
                  key={schedule.id}
                  schedule={schedule}
                />
              ))}
            </div>
          )}
      </section>
    </div>
  );
}
