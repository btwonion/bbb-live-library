import { apiFetch } from "./client";
import type { CreateTagRequest, Tag } from "./types";

export function listTags() {
  return apiFetch<Tag[]>("/api/tags");
}

export function createTag(data: CreateTagRequest) {
  return apiFetch<Tag>("/api/tags", {
    method: "POST",
    body: JSON.stringify(data),
  });
}
