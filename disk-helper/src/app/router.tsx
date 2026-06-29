import { lazy, Suspense, type ReactNode } from "react";
import { createBrowserRouter, Navigate } from "react-router-dom";
import { AppShell } from "./AppShell";
import { PageLoader } from "@/components/PageLoader";

const OverviewPage = lazy(() =>
  import("@/pages/overview/OverviewPage").then((m) => ({ default: m.OverviewPage })),
);
const ExplorerPage = lazy(() =>
  import("@/pages/explorer/ExplorerPage").then((m) => ({ default: m.ExplorerPage })),
);
const CleanupPage = lazy(() =>
  import("@/pages/cleanup/CleanupPage").then((m) => ({ default: m.CleanupPage })),
);
const QuarantinePage = lazy(() =>
  import("@/pages/quarantine/QuarantinePage").then((m) => ({ default: m.QuarantinePage })),
);
const AnalysisPage = lazy(() =>
  import("@/pages/analysis/AnalysisPage").then((m) => ({ default: m.AnalysisPage })),
);
const SettingsPage = lazy(() =>
  import("@/pages/settings/SettingsPage").then((m) => ({ default: m.SettingsPage })),
);
const AuditLogPage = lazy(() =>
  import("@/pages/audit/AuditLogPage").then((m) => ({ default: m.AuditLogPage })),
);

function LazyPage({ children }: { children: ReactNode }) {
  return <Suspense fallback={<PageLoader />}>{children}</Suspense>;
}

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true, element: <Navigate to="/overview" replace /> },
      {
        path: "overview",
        element: (
          <LazyPage>
            <OverviewPage />
          </LazyPage>
        ),
      },
      {
        path: "explorer",
        element: (
          <LazyPage>
            <ExplorerPage />
          </LazyPage>
        ),
      },
      {
        path: "cleanup",
        element: (
          <LazyPage>
            <CleanupPage />
          </LazyPage>
        ),
      },
      {
        path: "quarantine",
        element: (
          <LazyPage>
            <QuarantinePage />
          </LazyPage>
        ),
      },
      {
        path: "analysis",
        element: (
          <LazyPage>
            <AnalysisPage />
          </LazyPage>
        ),
      },
      {
        path: "settings",
        element: (
          <LazyPage>
            <SettingsPage />
          </LazyPage>
        ),
      },
      {
        path: "settings/audit",
        element: (
          <LazyPage>
            <AuditLogPage />
          </LazyPage>
        ),
      },
    ],
  },
]);
