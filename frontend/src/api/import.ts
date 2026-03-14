import { apiFetch } from "./client";
import type { ImportResult, ImportUrlRequest, Recording } from "./types";

export function triggerBbbImport() {
  return apiFetch<ImportResult>("/api/import/trigger", { method: "POST" });
}

export function importFromUrl(data: ImportUrlRequest) {
  return apiFetch<Recording>("/api/import/url", {
    method: "POST",
    body: JSON.stringify(data),
  });
}
