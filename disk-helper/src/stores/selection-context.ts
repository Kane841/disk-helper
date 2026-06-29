import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";
import type { ContextItem, FileNode } from "@/types";

interface SelectionState {
  source: "none" | "browse" | "cleanup";
  items: ContextItem[];
  selectedNode: FileNode | null;
  setFromBrowse: (node: FileNode) => void;
  setFromCleanup: (items: ContextItem[]) => void;
  clear: () => void;
}

export const useSelectionStore = create<SelectionState>()(
  persist(
    (set) => ({
      source: "none",
      items: [],
      selectedNode: null,
      setFromBrowse: (node) =>
        set({
          source: "browse",
          selectedNode: node,
          items: [
            {
              path: node.path,
              size_bytes: node.is_dir ? node.folder_size : node.size_bytes,
              is_dir: node.is_dir,
              risk: node.risk,
            },
          ],
        }),
      setFromCleanup: (items) =>
        set({ source: "cleanup", items, selectedNode: null }),
      clear: () => set({ source: "none", items: [], selectedNode: null }),
    }),
    {
      name: "disk-helper-selection",
      storage: createJSONStorage(() => sessionStorage),
      partialize: (state) => ({
        source: state.source,
        items: state.items,
        selectedNode: state.selectedNode,
      }),
    },
  ),
);
