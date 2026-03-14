import { useState, useRef, useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus, X } from "lucide-react";
import { toast } from "sonner";
import { assignCategories, assignTags } from "@/api/recordings";
import { listCategories } from "@/api/categories";
import { listTags } from "@/api/tags";
import type { Category, RecordingDetail, Tag } from "@/api/types";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

interface CategoryTagEditorProps {
  recording: RecordingDetail;
}

interface AutocompleteDropdownProps<T extends { id: string; name: string }> {
  items: T[];
  assignedIds: string[];
  onSelect: (item: T) => void;
  onClose: () => void;
}

function AutocompleteDropdown<T extends { id: string; name: string }>({
  items,
  assignedIds,
  onSelect,
  onClose,
}: AutocompleteDropdownProps<T>) {
  const [filter, setFilter] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (inputRef.current && !inputRef.current.closest("[data-dropdown]")?.contains(e.target as Node)) {
        onClose();
      }
    }
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [onClose]);

  const available = items.filter(
    (item) =>
      !assignedIds.includes(item.id) &&
      item.name.toLowerCase().includes(filter.toLowerCase()),
  );

  return (
    <div data-dropdown className="relative">
      <Input
        ref={inputRef}
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Escape") onClose();
        }}
        placeholder="Search..."
        className="h-7 w-40 text-xs"
      />
      {available.length > 0 && (
        <div className="absolute top-full z-10 mt-1 max-h-40 w-40 overflow-y-auto rounded-md border bg-popover p-1 shadow-md">
          {available.map((item) => (
            <button
              key={item.id}
              className="flex w-full cursor-pointer items-center rounded-sm px-2 py-1 text-left text-sm hover:bg-accent"
              onClick={() => {
                onSelect(item);
                onClose();
              }}
            >
              {item.name}
            </button>
          ))}
        </div>
      )}
      {available.length === 0 && filter && (
        <div className="absolute top-full z-10 mt-1 w-40 rounded-md border bg-popover p-2 text-xs text-muted-foreground shadow-md">
          No results
        </div>
      )}
    </div>
  );
}

export function CategoryTagEditor({ recording }: CategoryTagEditorProps) {
  const queryClient = useQueryClient();
  const [showCategoryPicker, setShowCategoryPicker] = useState(false);
  const [showTagPicker, setShowTagPicker] = useState(false);

  const { data: allCategories = [] } = useQuery({
    queryKey: ["categories"],
    queryFn: listCategories,
  });

  const { data: allTags = [] } = useQuery({
    queryKey: ["tags"],
    queryFn: listTags,
  });

  const categoryMutation = useMutation({
    mutationFn: (ids: string[]) => assignCategories(recording.id, { ids }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["recording", recording.id] });
      toast.success("Categories updated");
    },
  });

  const tagMutation = useMutation({
    mutationFn: (ids: string[]) => assignTags(recording.id, { ids }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["recording", recording.id] });
      toast.success("Tags updated");
    },
  });

  function addCategory(category: Category) {
    const ids = [...recording.categories.map((c) => c.id), category.id];
    categoryMutation.mutate(ids);
  }

  function removeCategory(categoryId: string) {
    const ids = recording.categories
      .filter((c) => c.id !== categoryId)
      .map((c) => c.id);
    categoryMutation.mutate(ids);
  }

  function addTag(tag: Tag) {
    const ids = [...recording.tags.map((t) => t.id), tag.id];
    tagMutation.mutate(ids);
  }

  function removeTag(tagId: string) {
    const ids = recording.tags
      .filter((t) => t.id !== tagId)
      .map((t) => t.id);
    tagMutation.mutate(ids);
  }

  return (
    <div className="space-y-4">
      {/* Categories */}
      <div>
        <h3 className="mb-2 text-sm font-medium">Categories</h3>
        <div className="flex flex-wrap items-center gap-2">
          {recording.categories.map((category) => (
            <Badge key={category.id} variant="secondary">
              {category.name}
              <button
                className="ml-1 rounded-full hover:bg-foreground/10"
                onClick={() => removeCategory(category.id)}
              >
                <X className="size-3" />
              </button>
            </Badge>
          ))}
          {showCategoryPicker ? (
            <AutocompleteDropdown
              items={allCategories}
              assignedIds={recording.categories.map((c) => c.id)}
              onSelect={addCategory}
              onClose={() => setShowCategoryPicker(false)}
            />
          ) : (
            <Button
              size="xs"
              variant="outline"
              onClick={() => setShowCategoryPicker(true)}
            >
              <Plus className="size-3" />
              Add
            </Button>
          )}
        </div>
      </div>

      {/* Tags */}
      <div>
        <h3 className="mb-2 text-sm font-medium">Tags</h3>
        <div className="flex flex-wrap items-center gap-2">
          {recording.tags.map((tag) => (
            <Badge key={tag.id} variant="outline">
              {tag.name}
              <button
                className="ml-1 rounded-full hover:bg-foreground/10"
                onClick={() => removeTag(tag.id)}
              >
                <X className="size-3" />
              </button>
            </Badge>
          ))}
          {showTagPicker ? (
            <AutocompleteDropdown
              items={allTags}
              assignedIds={recording.tags.map((t) => t.id)}
              onSelect={addTag}
              onClose={() => setShowTagPicker(false)}
            />
          ) : (
            <Button
              size="xs"
              variant="outline"
              onClick={() => setShowTagPicker(true)}
            >
              <Plus className="size-3" />
              Add
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
