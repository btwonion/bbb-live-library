import { useSearchParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { Grid3X3, List } from "lucide-react";
import { Button } from "@/components/ui/button";
import { FilterSidebar } from "@/components/FilterSidebar";
import { RecordingGrid } from "@/components/RecordingGrid";
import { Pagination } from "@/components/Pagination";
import { useDebounce } from "@/hooks/useDebounce";
import { listRecordings } from "@/api/recordings";
import { useState, useEffect } from "react";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";

const GRID_PER_PAGE = 12;
const LIST_PER_PAGE = 20;

export default function RecordingsPage() {
  useDocumentTitle("Recordings");

  const [searchParams, setSearchParams] = useSearchParams();

  const view = (searchParams.get("view") as "grid" | "list") || "grid";
  const page = Number(searchParams.get("page")) || 1;
  const categoryId = searchParams.get("category") || "";
  const urlSearch = searchParams.get("q") || "";

  const [searchInput, setSearchInput] = useState(urlSearch);
  const debouncedSearch = useDebounce(searchInput, 300);

  useEffect(() => {
    if (debouncedSearch !== urlSearch) {
      updateParam("q", debouncedSearch);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debouncedSearch]);

  const perPage = view === "grid" ? GRID_PER_PAGE : LIST_PER_PAGE;
  const hasFilters = !!(urlSearch || categoryId);

  const { data, isLoading } = useQuery({
    queryKey: ["recordings", { page, perPage, search: urlSearch, categoryId }],
    queryFn: () =>
      listRecordings({
        page,
        per_page: perPage,
        search: urlSearch || undefined,
        category_id: categoryId || undefined,
      }),
  });

  const totalPages = data ? Math.ceil(data.total / perPage) : 0;

  function updateParam(key: string, value: string) {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      if (value) {
        next.set(key, value);
      } else {
        next.delete(key);
      }
      if (key !== "page") next.delete("page");
      return next;
    });
  }

  function clearAll() {
    setSearchParams({});
    setSearchInput("");
  }

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Recordings</h1>
        <div className="flex items-center gap-1">
          <Button
            variant={view === "grid" ? "secondary" : "ghost"}
            size="icon-sm"
            onClick={() => updateParam("view", view === "grid" ? "" : "grid")}
          >
            <Grid3X3 className="size-4" />
          </Button>
          <Button
            variant={view === "list" ? "secondary" : "ghost"}
            size="icon-sm"
            onClick={() => updateParam("view", "list")}
          >
            <List className="size-4" />
          </Button>
        </div>
      </div>

      <div className="flex gap-6">
        <FilterSidebar
          search={searchInput}
          onSearchChange={setSearchInput}
          categoryId={categoryId}
          onCategoryChange={(v) => updateParam("category", v)}
          onClearAll={clearAll}
        />
        <div className="min-w-0 flex-1">
          <RecordingGrid
            recordings={data?.data}
            isLoading={isLoading}
            viewMode={view}
            hasFilters={hasFilters}
          />
        </div>
      </div>

      {totalPages > 1 && (
        <Pagination
          page={page}
          totalPages={totalPages}
          onPageChange={(p) => updateParam("page", String(p))}
        />
      )}
    </div>
  );
}
