import { useEffect } from "react";

const SUFFIX = "BBB Library";

export function useDocumentTitle(title: string) {
  useEffect(() => {
    document.title = `${title} — ${SUFFIX}`;
    return () => {
      document.title = SUFFIX;
    };
  }, [title]);
}
