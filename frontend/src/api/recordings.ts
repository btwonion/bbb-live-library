import { apiFetch } from "./client";
import type {
  AssignIdsRequest,
  PaginatedResponse,
  Recording,
  RecordingDetail,
  RecordingListParams,
  UpdateRecordingRequest,
} from "./types";

export function listRecordings(params?: RecordingListParams) {
  const query = new URLSearchParams();
  if (params) {
    for (const [key, value] of Object.entries(params)) {
      if (value !== undefined) query.set(key, String(value));
    }
  }
  const qs = query.toString();
  return apiFetch<PaginatedResponse<Recording>>(
    `/api/recordings${qs ? `?${qs}` : ""}`,
  );
}

export function getRecording(id: string) {
  return apiFetch<RecordingDetail>(`/api/recordings/${id}`);
}

export function updateRecording(id: string, data: UpdateRecordingRequest) {
  return apiFetch<Recording>(`/api/recordings/${id}`, {
    method: "POST",
    body: JSON.stringify(data),
  });
}

export function deleteRecording(id: string) {
  return apiFetch<void>(`/api/recordings/${id}`, { method: "DELETE" });
}

export function assignCategories(id: string, data: AssignIdsRequest) {
  return apiFetch<void>(`/api/recordings/${id}/categories`, {
    method: "POST",
    body: JSON.stringify(data),
  });
}

export function assignTags(id: string, data: AssignIdsRequest) {
  return apiFetch<void>(`/api/recordings/${id}/tags`, {
    method: "POST",
    body: JSON.stringify(data),
  });
}

export function getStreamUrl(id: string) {
  return `/api/recordings/${id}/stream`;
}

export function getThumbnailUrl(id: string) {
  return `/api/recordings/${id}/thumbnail`;
}
