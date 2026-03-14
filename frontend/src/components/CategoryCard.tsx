import { Edit2, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { Category } from "@/api/types";

interface CategoryCardProps {
  category: Category;
  onEdit: (category: Category) => void;
  onDelete: (category: Category) => void;
}

export function CategoryCard({ category, onEdit, onDelete }: CategoryCardProps) {
  return (
    <div className="flex items-start justify-between gap-4 rounded-lg border p-4">
      <div className="min-w-0 flex-1">
        <h3 className="truncate text-sm font-medium">{category.name}</h3>
        {category.description && (
          <p className="mt-1 text-xs text-muted-foreground">
            {category.description}
          </p>
        )}
      </div>
      <div className="flex shrink-0 items-center gap-1">
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={() => onEdit(category)}
        >
          <Edit2 className="size-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={() => onDelete(category)}
        >
          <Trash2 className="size-4" />
        </Button>
      </div>
    </div>
  );
}
