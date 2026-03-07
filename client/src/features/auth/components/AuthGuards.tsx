import { Navigate, useLocation } from 'react-router-dom';
import { PageLoader } from '@/components/shared/LoadingSpinner';
import { ROUTES, STORAGE_KEYS } from '@/lib/constants';
import { useAuthStore } from '@/stores';
import { useInitAuth } from '../hooks/useAuth';

interface AuthGuardProps {
  children: React.ReactNode;
}

/**
 * Guard that requires valid API key authentication.
 * Redirects to login if not authenticated.
 */
export function RequireAuth({ children }: AuthGuardProps) {
  const location = useLocation();
  const apiKey = localStorage.getItem(STORAGE_KEYS.AUTH_TOKEN);
  const isLoading = useAuthStore((state) => state.isLoading);
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  const { isError } = useInitAuth();

  if (!apiKey) {
    return <Navigate to={ROUTES.LOGIN} state={{ from: location }} replace />;
  }

  if (isLoading) {
    return <PageLoader />;
  }

  if (isError || !isAuthenticated) {
    localStorage.removeItem(STORAGE_KEYS.AUTH_TOKEN);
    return <Navigate to={ROUTES.LOGIN} state={{ from: location }} replace />;
  }

  return <>{children}</>;
}

/**
 * Guard that requires guest (no auth).
 * Redirects to dashboard if already authenticated.
 */
export function RequireGuest({ children }: AuthGuardProps) {
  const apiKey = localStorage.getItem(STORAGE_KEYS.AUTH_TOKEN);
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);

  if (apiKey && isAuthenticated) {
    return <Navigate to={ROUTES.DASHBOARD} replace />;
  }

  return <>{children}</>;
}
