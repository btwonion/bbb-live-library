import { apiFetch } from "./client";
import type {
  Category,
  CreateCategoryRequest,
  UpdateCategoryRequest,
} from "./types";

export function listCategories() {
  return apiFetch<Category[]>("/api/categories");
}

export function createCategory(data: CreateCategoryRequest) {
  return apiFetch<Category>("/api/categories", {
    method: "POST",
    body: JSON.stringify(data),
  });
}

export function updateCategory(id: string, data: UpdateCategoryRequest) {
  return apiFetch<Category>(`/api/categories/${id}`, {
    method: "PUT",
    body: JSON.stringify(data),
  });
}

export function deleteCategory(id: string) {
  return apiFetch<void>(`/api/categories/${id}`, { method: "DELETE" });
}
