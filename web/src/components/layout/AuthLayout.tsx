import { Outlet, Navigate } from 'react-router-dom';
import { MessageSquare } from 'lucide-react';
import { useAuthStore } from '@/store/authStore';

export function AuthLayout() {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);

  if (isAuthenticated) {
    return <Navigate to="/dashboard" replace />;
  }

  return (
    <div className="min-h-screen bg-bg-light dark:bg-bg-dark flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        {/* Logo */}
        <div className="text-center mb-8">
          <div className="h-16 w-16 rounded-2xl bg-primary-500 flex items-center justify-center mx-auto mb-4">
            <MessageSquare className="h-8 w-8 text-white" />
          </div>
          <h1 className="text-2xl font-bold text-text-light dark:text-text-dark">
            WAS Admin
          </h1>
          <p className="text-text-muted-light dark:text-text-muted-dark mt-1">
            WhatsApp Automation Server
          </p>
        </div>

        <Outlet />
      </div>
    </div>
  );
}
