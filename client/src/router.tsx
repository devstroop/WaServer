import { lazy, Suspense } from 'react';
import { createBrowserRouter, Navigate, Outlet } from 'react-router-dom';
import { RequireAuth, RequireGuest } from '@/features/auth';
import { PageLoader } from '@/components/shared/LoadingSpinner';
import { ROUTES } from '@/lib/constants';

const DashboardPage = lazy(() => import('@/pages/DashboardPage').then((m) => ({ default: m.DashboardPage })));
const InstancesPage = lazy(() => import('@/pages/InstancesPage').then((m) => ({ default: m.InstancesPage })));
const InstanceDetailPage = lazy(() => import('@/pages/InstanceDetailPage').then((m) => ({ default: m.InstanceDetailPage })));
const MessagesPage = lazy(() => import('@/pages/MessagesPage').then((m) => ({ default: m.MessagesPage })));
const SettingsPage = lazy(() => import('@/pages/SettingsPage').then((m) => ({ default: m.SettingsPage })));
const LoginPage = lazy(() => import('@/pages/LoginPage').then((m) => ({ default: m.LoginPage })));
const NotFoundPage = lazy(() => import('@/pages/NotFoundPage').then((m) => ({ default: m.NotFoundPage })));

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
      <Suspense fallback={<PageLoader />}>
        <Outlet />
      </Suspense>
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
    children: [
      { path: ROUTES.DASHBOARD, element: <DashboardPage /> },
      { path: ROUTES.INSTANCES, element: <InstancesPage /> },
      { path: '/instances/:id', element: <InstanceDetailPage /> },
      { path: ROUTES.MESSAGES, element: <MessagesPage /> },
      { path: ROUTES.SETTINGS, element: <SettingsPage /> },
    ],
  },
  {
    element: <GuestLayout />,
    children: [
      { path: ROUTES.LOGIN, element: <LoginPage /> },
    ],
  },
  {
    element: <SuspenseLayout />,
    children: [
      { path: '*', element: <NotFoundPage /> },
    ],
  },
]);
