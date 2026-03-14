import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider, MutationCache } from "@tanstack/react-query";
import { Toaster, toast } from "sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import "./index.css";
import App from "./App.tsx";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      retry: 1,
    },
  },
  mutationCache: new MutationCache({
    onError: (error) => {
      toast.error(error instanceof Error ? error.message : "An error occurred");
    },
  }),
});

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter>
      <QueryClientProvider client={queryClient}>
        <TooltipProvider>
          <App />
          <Toaster richColors position="bottom-right" />
        </TooltipProvider>
      </QueryClientProvider>
    </BrowserRouter>
  </StrictMode>,
);
