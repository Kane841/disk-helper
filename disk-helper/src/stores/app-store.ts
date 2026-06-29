import { create } from "zustand";
import type { ScanStatus } from "@/types";

export { useSelectionStore } from "./selection-context";

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
