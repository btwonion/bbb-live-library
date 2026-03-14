import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { createCategory, updateCategory } from "@/api/categories";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import type {
  Category,
  CreateCategoryRequest,
  UpdateCategoryRequest,
} from "@/api/types";

interface CategoryFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  category?: Category;
}

interface FormFieldsProps {
  category?: Category;
  onOpenChange: (open: boolean) => void;
}

function FormFields({ category, onOpenChange }: FormFieldsProps) {
  const isEdit = !!category;
  const queryClient = useQueryClient();

  const [name, setName] = useState(category?.name ?? "");
  const [description, setDescription] = useState(category?.description ?? "");

  const createMutation = useMutation({
    mutationFn: (data: CreateCategoryRequest) => createCategory(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["categories"] });
      toast.success("Category created");
      onOpenChange(false);
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: UpdateCategoryRequest) =>
      updateCategory(category!.id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["categories"] });
      toast.success("Category updated");
      onOpenChange(false);
    },
  });

  const isPending = createMutation.isPending || updateMutation.isPending;
  const error = createMutation.error || updateMutation.error;

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const data = { name, description: description || undefined };
    if (isEdit) {
      updateMutation.mutate(data);
    } else {
      createMutation.mutate(data);
    }
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-3">
      <div className="space-y-1">
        <label htmlFor="cf-name" className="text-xs font-medium">
          Name *
        </label>
        <Input
          id="cf-name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          required
        />
      </div>

      <div className="space-y-1">
        <label htmlFor="cf-desc" className="text-xs font-medium">
          Description
        </label>
        <Textarea
          id="cf-desc"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          rows={3}
          placeholder="Optional description"
        />
      </div>

      {error && (
        <p className="text-xs text-destructive">
          {error instanceof Error ? error.message : "An error occurred"}
        </p>
      )}

      <DialogFooter>
        <Button
          type="button"
          variant="outline"
          onClick={() => onOpenChange(false)}
        >
          Cancel
        </Button>
        <Button type="submit" disabled={isPending}>
          {isPending
            ? isEdit
              ? "Saving..."
              : "Creating..."
            : isEdit
              ? "Save Changes"
              : "Create Category"}
        </Button>
      </DialogFooter>
    </form>
  );
}

export function CategoryForm({
  open,
  onOpenChange,
  category,
}: CategoryFormProps) {
  const isEdit = !!category;
  const formKey = category?.id ?? "new";

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>
            {isEdit ? "Edit Category" : "New Category"}
          </DialogTitle>
          <DialogDescription>
            {isEdit
              ? "Update the category details."
              : "Create a new category for organizing recordings."}
          </DialogDescription>
        </DialogHeader>
        <FormFields
          key={formKey}
          category={category}
          onOpenChange={onOpenChange}
        />
      </DialogContent>
    </Dialog>
  );
}
