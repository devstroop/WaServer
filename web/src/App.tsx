import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import { ToastProvider } from '@devstroop/react-uikit';
import { AuthProvider } from './hooks/useAuth';
import Layout from './components/Layout';
import { AuthGuard, PublicOnlyGuard, RoleGuard } from './components/guards';
import Home from './pages/Home';
import Login from './pages/Login';
import Register from './pages/Register';
import Dashboard from './pages/Dashboard';
import Instances from './pages/Instances';
import AdminDashboard from './pages/AdminDashboard';
import InstanceDetail from './pages/InstanceDetail';
import UsersList from './pages/UsersList';
import UserDetail from './pages/UserDetail';
import ApiKeys from './pages/ApiKeys';
import AdminUserTokens from './pages/AdminUserTokens';
import AdminUserAssignments from './pages/AdminUserAssignments';

function NotImplemented() {
  return <div className="p-8 text-center text-zinc-500">Not implemented</div>;
}

export default function App() {
  return (
    <BrowserRouter>
      <ToastProvider position="top-right">
        <AuthProvider>
          <Routes>
            {/* Public */}
            <Route path="/" element={<Home />} />

            {/* Auth – public only */}
            <Route
              path="/auth/login"
              element={
                <PublicOnlyGuard>
                  <Login />
                </PublicOnlyGuard>
              }
            />
            <Route
              path="/auth/register"
              element={
                <PublicOnlyGuard>
                  <Register />
                </PublicOnlyGuard>
              }
            />
            {/* Legacy compat redirects */}
            <Route path="/login" element={<Navigate to="/auth/login" replace />} />
            <Route path="/register" element={<Navigate to="/auth/register" replace />} />

            {/* Protected user zone */}
            <Route
              element={
                <AuthGuard>
                  <Layout />
                </AuthGuard>
              }
            >
              <Route path="/dashboard" element={<Dashboard />} />
              <Route path="/dashboard/instances" element={<Instances />} />
              <Route path="/dashboard/instances/:id" element={<InstanceDetail />} />
              <Route path="/settings" element={<ApiKeys />} />
              <Route path="/settings/api-keys" element={<ApiKeys />} />
              {/* Legacy /app compat */}
              <Route path="/app" element={<Navigate to="/dashboard" replace />} />
              <Route path="/app/instances" element={<Navigate to="/dashboard/instances" replace />} />
            </Route>

            {/* Protected admin zone */}
            <Route
              element={
                <RoleGuard>
                  <Layout />
                </RoleGuard>
              }
            >
              <Route path="/admin" element={<NotImplemented />} />
              <Route path="/admin/dashboard" element={<AdminDashboard />} />
              <Route path="/admin/metrics" element={<NotImplemented />} />
              <Route path="/admin/instances" element={<Instances admin />} />
              <Route path="/admin/instances/:id" element={<InstanceDetail />} />
              <Route path="/admin/users" element={<UsersList />} />
              <Route path="/admin/users/:id" element={<UserDetail />} />
              <Route path="/admin/users/:id/tokens" element={<AdminUserTokens />} />
              <Route path="/admin/users/:id/instances" element={<AdminUserAssignments />} />
              <Route path="/admin/users/:id/assignments" element={<AdminUserAssignments />} />
            </Route>

            {/* Fallback */}
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </AuthProvider>
      </ToastProvider>
    </BrowserRouter>
  );
}
