import { useEffect, useRef } from "react";
import { useTheme } from "next-themes";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";

/** Apply persisted theme from backend settings once on startup. */
export function ThemeSync() {
  const { setTheme } = useTheme();
  const initialSyncDone = useRef(false);
  const { data: settings } = useQuery({
    queryKey: ["settings"],
    queryFn: () => api.configGet(),
    retry: false,
  });

  useEffect(() => {
    if (!settings?.theme || initialSyncDone.current) return;
    initialSyncDone.current = true;
    setTheme(settings.theme);
  }, [settings?.theme, setTheme]);

  return null;
}
