import { apiFetch } from "./client";
import type { StatsResponse } from "./types";

export function getStats() {
  return apiFetch<StatsResponse>("/api/stats");
}
