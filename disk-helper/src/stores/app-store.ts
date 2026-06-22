import { create } from "zustand";
import type { ContextItem, FileNode, ScanStatus } from "@/types";

interface SelectionState {
  source: "none" | "browse" | "cleanup";
  items: ContextItem[];
  selectedNode: FileNode | null;
  setFromBrowse: (node: FileNode) => void;
  setFromCleanup: (items: ContextItem[]) => void;
  clear: () => void;
}

export const useSelectionStore = create<SelectionState>((set) => ({
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
}));

interface ScanState {
  status: ScanStatus;
  progress: number;
  scannedFiles: number;
  setStatus: (s: ScanStatus) => void;
  setProgress: (p: number) => void;
  setScannedFiles: (n: number) => void;
}

export const useScanStore = create<ScanState>((set) => ({
  status: "completed",
  progress: 100,
  scannedFiles: 284521,
  setStatus: (status) => set({ status }),
  setProgress: (progress) => set({ progress }),
  setScannedFiles: (scannedFiles) => set({ scannedFiles }),
}));

interface ToastState {
  message: string | null;
  show: (message: string) => void;
  hide: () => void;
}

export const useToastStore = create<ToastState>((set) => ({
  message: null,
  show: (message) => {
    set({ message });
    setTimeout(() => set({ message: null }), 3200);
  },
  hide: () => set({ message: null }),
}));
