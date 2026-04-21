import { apiFetch } from "./client";

interface TimezoneResponse {
  timezone: string;
}

export function getTimezone() {
  return apiFetch<TimezoneResponse>("/api/settings/timezone");
}
