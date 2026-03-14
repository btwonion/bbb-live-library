import { apiFetch } from "./client";
import type {
  CreateScheduleRequest,
  PaginatedResponse,
  PaginationParams,
  Schedule,
  UpdateScheduleRequest,
} from "./types";

export function listSchedules(params?: PaginationParams) {
  const query = new URLSearchParams();
  if (params) {
    for (const [key, value] of Object.entries(params)) {
      if (value !== undefined) query.set(key, String(value));
    }
  }
  const qs = query.toString();
  return apiFetch<PaginatedResponse<Schedule>>(
    `/api/schedules${qs ? `?${qs}` : ""}`,
  );
}

export function getSchedule(id: string) {
  return apiFetch<Schedule>(`/api/schedules/${id}`);
}

export function createSchedule(data: CreateScheduleRequest) {
  return apiFetch<Schedule>("/api/schedules", {
    method: "POST",
    body: JSON.stringify(data),
  });
}

export function updateSchedule(id: string, data: UpdateScheduleRequest) {
  return apiFetch<Schedule>(`/api/schedules/${id}`, {
    method: "PUT",
    body: JSON.stringify(data),
  });
}

export function deleteSchedule(id: string) {
  return apiFetch<void>(`/api/schedules/${id}`, { method: "DELETE" });
}
