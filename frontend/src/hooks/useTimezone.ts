import { useQuery } from "@tanstack/react-query";
import { getTimezone } from "@/api/settings";

export function useTimezone() {
  const { data } = useQuery({
    queryKey: ["settings", "timezone"],
    queryFn: getTimezone,
    staleTime: Infinity,
  });
  return data?.timezone ?? "UTC";
}
