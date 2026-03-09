import { lazy, Suspense } from 'react';
import { createBrowserRouter, Navigate, Outlet } from 'react-router-dom';
import { RequireAuth, RequireGuest } from '@/features/auth';
import { PageLoader } from '@/components/shared/LoadingSpinner';
import { RouteErrorFallback } from '@/components/shared/ErrorBoundary';
import { DashboardLayout } from '@/layouts';
import { ROUTES } from '@/lib/constants';

// New Pages
const DashboardPage = lazy(() =>
  import('@/pages/dashboard').then((m) => ({ default: m.DashboardPage }))
);
const ServersPage = lazy(() =>
  import('@/pages/servers').then((m) => ({ default: m.ServersPage }))
);
const SessionsPage = lazy(() =>
  import('@/pages/sessions').then((m) => ({ default: m.SessionsPage }))
);
const MessagesPage = lazy(() =>
  import('@/pages/messages').then((m) => ({ default: m.MessagesPage }))
);
const CampaignsPage = lazy(() =>
  import('@/pages/campaigns').then((m) => ({ default: m.CampaignsPage }))
);
const ContactsPage = lazy(() =>
  import('@/pages/contacts').then((m) => ({ default: m.ContactsPage }))
);
const AnalyticsPage = lazy(() =>
  import('@/pages/analytics').then((m) => ({ default: m.AnalyticsPage }))
);
const ApiKeysPage = lazy(() =>
  import('@/pages/apiKeys').then((m) => ({ default: m.ApiKeysPage }))
);
const LogsPage = lazy(() =>
  import('@/pages/logs').then((m) => ({ default: m.LogsPage }))
);
const SettingsPage = lazy(() =>
  import('@/pages/settings').then((m) => ({ default: m.SettingsPage }))
);
const LoginPage = lazy(() =>
  import('@/pages/LoginPage').then((m) => ({ default: m.LoginPage }))
);
const NotFoundPage = lazy(() =>
  import('@/pages/NotFoundPage').then((m) => ({ default: m.NotFoundPage }))
);

function SuspenseLayout() {
  return (
    <Suspense fallback={<PageLoader />}>
      <Outlet />
    </Suspense>
  );
}

function ProtectedLayout() {
  return (
    <RequireAuth>
      <DashboardLayout>
        <Suspense fallback={<PageLoader />}>
          <Outlet />
        </Suspense>
      </DashboardLayout>
    </RequireAuth>
  );
}

function GuestLayout() {
  return (
    <RequireGuest>
      <Suspense fallback={<PageLoader />}>
        <Outlet />
      </Suspense>
    </RequireGuest>
  );
}

export const router = createBrowserRouter([
  {
    path: '/',
    element: <Navigate to={ROUTES.DASHBOARD} replace />,
  },
  {
    element: <ProtectedLayout />,
    errorElement: <RouteErrorFallback />,
    children: [
      { path: ROUTES.DASHBOARD, element: <DashboardPage /> },
      { path: ROUTES.SERVERS, element: <ServersPage /> },
      { path: ROUTES.SESSIONS, element: <SessionsPage /> },
      { path: ROUTES.MESSAGES, element: <MessagesPage /> },
      { path: ROUTES.CAMPAIGNS, element: <CampaignsPage /> },
      { path: ROUTES.CONTACTS, element: <ContactsPage /> },
      { path: ROUTES.ANALYTICS, element: <AnalyticsPage /> },
      { path: ROUTES.API_KEYS, element: <ApiKeysPage /> },
      { path: ROUTES.LOGS, element: <LogsPage /> },
      { path: ROUTES.SETTINGS, element: <SettingsPage /> },
    ],
  },
  {
    element: <GuestLayout />,
    errorElement: <RouteErrorFallback />,
    children: [{ path: ROUTES.LOGIN, element: <LoginPage /> }],
  },
  {
    element: <SuspenseLayout />,
    children: [{ path: '*', element: <NotFoundPage /> }],
  },
]);
