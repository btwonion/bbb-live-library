import { apiFetch } from "./client";
import type { ImportPublicBbbRequest, ImportUrlRequest, Recording } from "./types";

export function importFromUrl(data: ImportUrlRequest) {
  return apiFetch<Recording>("/api/import/url", {
    method: "POST",
    body: JSON.stringify(data),
  });
}

export function importPublicBbb(data: ImportPublicBbbRequest) {
  return apiFetch<Recording>("/api/import/bbb-public", {
    method: "POST",
    body: JSON.stringify(data),
  });
}
