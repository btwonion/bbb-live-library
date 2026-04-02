export interface Recording {
  id: string;
  title: string;
  description: string | null;
  file_path: string;
  thumbnail_path: string | null;
  duration_seconds: number | null;
  file_size_bytes: number | null;
  format: string;
  source: string;
  bbb_meeting_id: string | null;
  schedule_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface RecordingDetail extends Recording {
  categories: Category[];
}

export interface Category {
  id: string;
  name: string;
  description: string | null;
  created_at: string;
}

export interface Schedule {
  id: string;
  title: string;
  start_time: string;
  end_time: string | null;
  recurrence: string | null;
  enabled: boolean;
  created_at: string;
  updated_at: string;
  stream_url: string;
  room_url: string;
  bot_name: string;
  status: string;
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  per_page: number;
}

export interface StatsResponse {
  recording_count: number;
  total_duration_seconds: number;
  total_size_bytes: number;
  by_category: CategoryCount[];
}

export interface CategoryCount {
  category_name: string;
  count: number;
}

export interface ImportPublicBbbRequest {
  url: string;
  record_id?: string;
  title?: string;
}

export interface UpdateRecordingRequest {
  title?: string;
  description?: string;
}

export interface AssignIdsRequest {
  ids: string[];
}

export interface CreateScheduleRequest {
  title: string;
  stream_url?: string;
  start_time: string;
  end_time?: string;
  recurrence?: string;
  room_url?: string;
  bot_name?: string;
}

export interface UpdateScheduleRequest {
  title?: string;
  stream_url?: string;
  start_time?: string;
  end_time?: string;
  recurrence?: string;
  enabled?: boolean;
  room_url?: string;
  bot_name?: string;
}

export interface CreateCategoryRequest {
  name: string;
  description?: string;
}

export interface UpdateCategoryRequest {
  name?: string;
  description?: string;
}

export interface ImportUrlRequest {
  url: string;
  title?: string;
}

export interface RecordingListParams {
  page?: number;
  per_page?: number;
  search?: string;
  category_id?: string;
}

export interface PaginationParams {
  page?: number;
  per_page?: number;
}
