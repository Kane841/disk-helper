import { createBrowserRouter, Navigate } from "react-router-dom";
import { AppShell } from "./AppShell";
import { OverviewPage } from "@/pages/overview/OverviewPage";
import { ExplorerPage } from "@/pages/explorer/ExplorerPage";
import { CleanupPage } from "@/pages/cleanup/CleanupPage";
import { QuarantinePage } from "@/pages/quarantine/QuarantinePage";
import { AnalysisPage } from "@/pages/analysis/AnalysisPage";
import { SettingsPage } from "@/pages/settings/SettingsPage";
import { AuditLogPage } from "@/pages/audit/AuditLogPage";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true, element: <Navigate to="/overview" replace /> },
      { path: "overview", element: <OverviewPage /> },
      { path: "explorer", element: <ExplorerPage /> },
      { path: "cleanup", element: <CleanupPage /> },
      { path: "quarantine", element: <QuarantinePage /> },
      { path: "analysis", element: <AnalysisPage /> },
      { path: "settings", element: <SettingsPage /> },
      { path: "settings/audit", element: <AuditLogPage /> },
    ],
  },
]);
