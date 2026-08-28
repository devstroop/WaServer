import { Navigate, useLocation } from 'react-router-dom';
import { Skeleton } from '@devstroop/react-uikit';
import { getToken } from '../api/client';
import { useAuth } from '../hooks/useAuth';
import type { JSX } from 'react';

export default function ProtectedRoute({ children }: { children: JSX.Element }) {
  const { user, loading, isAuthenticated } = useAuth();
  const location = useLocation();

  if (loading) {
    return (
      <div className="p-8 space-y-3" aria-busy="true" aria-label="Loading">
        <Skeleton variant="rect" width="100%" height={32} />
        <Skeleton variant="rect" width="100%" height={120} />
      </div>
    );
  }

  const token = getToken();
  if (!token || !isAuthenticated || !user || user.is_active === false) {
    const next = encodeURIComponent(location.pathname + location.search);
    return <Navigate to={`/auth/login?next=${next}`} replace />;
  }

  return children;
}
