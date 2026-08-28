import { Navigate } from 'react-router-dom';
import { Skeleton } from '@devstroop/react-uikit';
import { useAuth } from '../../hooks/useAuth';
import type { ReactNode } from 'react';

export function PublicOnlyGuard({ children }: { children: ReactNode }) {
  const { user, loading, isAuthenticated, isAdmin } = useAuth();

  if (loading) {
    return (
      <div className="p-8 space-y-3" aria-busy="true" aria-label="Loading">
        <Skeleton variant="rect" width="100%" height={32} />
        <Skeleton variant="rect" width="40%" height={16} />
      </div>
    );
  }

  if (isAuthenticated && user && user.is_active !== false) {
    const target = isAdmin ? '/admin/dashboard' : '/dashboard';
    return <Navigate to={target} replace />;
  }

  return <>{children}</>;
}

export default PublicOnlyGuard;
