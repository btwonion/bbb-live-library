import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, FolderOpen } from "lucide-react";
import { toast } from "sonner";
import { listCategories, deleteCategory } from "@/api/categories";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { CategoryCard } from "@/components/CategoryCard";
import { CategoryForm } from "@/components/CategoryForm";
import type { Category } from "@/api/types";

export default function CategoriesPage() {
  useDocumentTitle("Categories");

  const queryClient = useQueryClient();

  const [formOpen, setFormOpen] = useState(false);
  const [editingCategory, setEditingCategory] = useState<
    Category | undefined
  >();
  const [deletingCategory, setDeletingCategory] = useState<Category | null>(
    null,
  );

  const { data: categories, isLoading } = useQuery({
    queryKey: ["categories"],
    queryFn: listCategories,
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteCategory(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["categories"] });
      toast.success("Category deleted");
      setDeletingCategory(null);
    },
  });

  function handleEdit(category: Category) {
    setEditingCategory(category);
    setFormOpen(true);
  }

  function handleCreate() {
    setEditingCategory(undefined);
    setFormOpen(true);
  }

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Categories</h1>
        <Button onClick={handleCreate}>
          <Plus className="size-4" />
          New Category
        </Button>
      </div>

      {isLoading && (
        <div className="space-y-3">
          {Array.from({ length: 3 }).map((_, i) => (
            <Skeleton key={i} className="h-16 w-full rounded-lg" />
          ))}
        </div>
      )}

      {!isLoading && categories && categories.length === 0 && (
        <div className="flex flex-col items-center gap-3 rounded-lg border border-dashed p-12 text-center">
          <FolderOpen className="size-10 text-muted-foreground" />
          <div>
            <p className="font-medium">No categories yet</p>
            <p className="text-sm text-muted-foreground">
              Create your first category to organize recordings.
            </p>
          </div>
          <Button onClick={handleCreate}>
            <Plus className="size-4" />
            New Category
          </Button>
        </div>
      )}

      {!isLoading && categories && categories.length > 0 && (
        <div className="space-y-3">
          {categories.map((category) => (
            <CategoryCard
              key={category.id}
              category={category}
              onEdit={handleEdit}
              onDelete={setDeletingCategory}
            />
          ))}
        </div>
      )}

      <CategoryForm
        open={formOpen}
        onOpenChange={setFormOpen}
        category={editingCategory}
      />

      <AlertDialog
        open={!!deletingCategory}
        onOpenChange={(open) => {
          if (!open) setDeletingCategory(null);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete category?</AlertDialogTitle>
            <AlertDialogDescription>
              This will permanently delete &ldquo;{deletingCategory?.name}
              &rdquo;. Recordings in this category will not be deleted.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={() => {
                if (deletingCategory)
                  deleteMutation.mutate(deletingCategory.id);
              }}
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending ? "Deleting..." : "Delete"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
