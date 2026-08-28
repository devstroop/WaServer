import { Navigate, useLocation } from 'react-router-dom';
import { Alert, Skeleton } from '@devstroop/react-uikit';
import { getToken } from '../../api/client';
import { useAuth } from '../../hooks/useAuth';
import type { ReactNode } from 'react';

export function RoleGuard({ children }: { children: ReactNode }) {
  const { user, loading, isAuthenticated, isAdmin } = useAuth();
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

  if (!isAdmin) {
    return (
      <div className="p-8">
        <Alert tone="danger" title="Forbidden" variant="soft">
          Admin access required. Your account does not have administrator privileges.
        </Alert>
      </div>
    );
  }

  return <>{children}</>;
}

export default RoleGuard;
