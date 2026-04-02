import { useQuery } from "@tanstack/react-query";
import { Search, X } from "lucide-react";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { listCategories } from "@/api/categories";

interface FilterSidebarProps {
  search: string;
  onSearchChange: (value: string) => void;
  categoryId: string;
  onCategoryChange: (value: string) => void;
  onClearAll: () => void;
}

export function FilterSidebar({
  search,
  onSearchChange,
  categoryId,
  onCategoryChange,
  onClearAll,
}: FilterSidebarProps) {
  const { data: categories } = useQuery({
    queryKey: ["categories"],
    queryFn: listCategories,
  });

  const hasFilters = search || categoryId;

  return (
    <aside className="hidden w-60 shrink-0 flex-col gap-5 md:flex">
      <div className="relative">
        <Search className="absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          placeholder="Search recordings..."
          value={search}
          onChange={(e) => onSearchChange(e.target.value)}
          className="pl-8"
        />
      </div>

      {categories && categories.length > 0 && (
        <div className="flex flex-col gap-1.5">
          <label className="text-xs font-medium text-muted-foreground">
            Category
          </label>
          <Select
            value={categoryId || "all"}
            onValueChange={(v) => onCategoryChange(v === "all" || !v ? "" : v)}
          >
            <SelectTrigger className="w-full">
              <SelectValue>
                {(value: string) => {
                  if (!value || value === "all") return "All categories";
                  return categories?.find((c) => c.id === value)?.name ?? value;
                }}
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All categories</SelectItem>
              {categories.map((c) => (
                <SelectItem key={c.id} value={c.id}>
                  {c.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      )}

      {hasFilters && (
        <Button variant="ghost" size="sm" onClick={onClearAll} className="w-fit">
          <X className="size-3.5" />
          Clear filters
        </Button>
      )}
    </aside>
  );
}
