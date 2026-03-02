import { Outlet, Navigate } from 'react-router-dom';
import { Sidebar } from './Sidebar';
import { useAuthStore } from '@/store/authStore';

export function AdminLayout() {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return (
    <div className="min-h-screen bg-bg-light dark:bg-bg-dark">
      <Sidebar />
      <main className="lg:ml-64 min-h-screen">
        <Outlet />
      </main>
    </div>
  );
}
