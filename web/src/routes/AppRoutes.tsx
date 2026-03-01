import { lazy, Suspense } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { AdminLayout, AuthLayout } from '@/components/layout';
import { Skeleton } from '@/components/ui';

// Lazy load pages
const LoginPage = lazy(() => import('@/pages/auth/LoginPage').then(m => ({ default: m.LoginPage })));
const RegisterPage = lazy(() => import('@/pages/auth/RegisterPage').then(m => ({ default: m.RegisterPage })));
const DashboardPage = lazy(() => import('@/pages/dashboard/DashboardPage').then(m => ({ default: m.DashboardPage })));
const InstancesListPage = lazy(() => import('@/pages/instances/InstancesListPage').then(m => ({ default: m.InstancesListPage })));
const NewInstancePage = lazy(() => import('@/pages/instances/NewInstancePage').then(m => ({ default: m.NewInstancePage })));
const InstanceDetailPage = lazy(() => import('@/pages/instances/InstanceDetailPage').then(m => ({ default: m.InstanceDetailPage })));
const LinkInstancePage = lazy(() => import('@/pages/instances/LinkInstancePage').then(m => ({ default: m.LinkInstancePage })));
const UsersListPage = lazy(() => import('@/pages/users/UsersListPage').then(m => ({ default: m.UsersListPage })));
const NewUserPage = lazy(() => import('@/pages/users/NewUserPage').then(m => ({ default: m.NewUserPage })));
const SettingsPage = lazy(() => import('@/pages/settings/SettingsPage').then(m => ({ default: m.SettingsPage })));

function PageLoader() {
  return (
    <div className="p-6">
      <Skeleton className="h-8 w-48 mb-4" />
      <Skeleton className="h-64 w-full" />
    </div>
  );
}

export function AppRoutes() {
  return (
    <Suspense fallback={<PageLoader />}>
      <Routes>
        {/* Auth routes */}
        <Route element={<AuthLayout />}>
          <Route path="/login" element={<LoginPage />} />
          <Route path="/register" element={<RegisterPage />} />
        </Route>

        {/* Protected routes */}
        <Route element={<AdminLayout />}>
          <Route path="/dashboard" element={<DashboardPage />} />
          
          {/* Instances */}
          <Route path="/instances" element={<InstancesListPage />} />
          <Route path="/instances/new" element={<NewInstancePage />} />
          <Route path="/instances/:instanceId" element={<InstanceDetailPage />} />
          <Route path="/instances/:instanceId/link" element={<LinkInstancePage />} />
          
          {/* Users */}
          <Route path="/users" element={<UsersListPage />} />
          <Route path="/users/new" element={<NewUserPage />} />
          
          {/* Settings */}
          <Route path="/settings" element={<SettingsPage />} />
        </Route>

        {/* Redirects */}
        <Route path="/" element={<Navigate to="/dashboard" replace />} />
        <Route path="*" element={<Navigate to="/dashboard" replace />} />
      </Routes>
    </Suspense>
  );
}
