import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { queryKeys } from '@/lib/query-client';
import { authService } from '@/services/auth.service';
import { healthService } from '@/services/health.service';
import { instanceService } from '@/services/instance.service';
import { ROUTES } from '@/lib/constants';
import { useToast } from '@/hooks/useToast';
import { useAuthStore } from '@/stores';

/**
 * Hook to check server health and validate API key on app load
 */
export function useInitAuth() {
  const setLoading = useAuthStore((state) => state.setLoading);
  const setServerVersion = useAuthStore((state) => state.setServerVersion);
  const authenticate = useAuthStore((state) => state.authenticate);
  const logout = useAuthStore((state) => state.logout);

  return useQuery({
    queryKey: queryKeys.auth.user,
    queryFn: async () => {
      setLoading(true);
      try {
        // Check server health first (no auth required)
        const health = await healthService.check();
        setServerVersion(health.version);

        // If we have an API key stored, validate it
        const storedKey = authService.getApiKey();
        if (storedKey) {
          try {
            // Try to list instances to validate the key
            await instanceService.list();
            authenticate(storedKey);
            return { authenticated: true, serverVersion: health.version };
          } catch {
            // Key is invalid, clear it
            logout();
            return { authenticated: false, serverVersion: health.version };
          }
        }

        return { authenticated: false, serverVersion: health.version };
      } finally {
        setLoading(false);
      }
    },
    retry: false,
    staleTime: 5 * 60 * 1000,
  });
}

/**
 * Hook to authenticate with an API key
 */
export function useAuthenticate() {
  const navigate = useNavigate();
  const { toast } = useToast();
  const authenticate = useAuthStore((state) => state.authenticate);
  const setValidating = useAuthStore((state) => state.setValidating);

  return useMutation({
    mutationFn: async (apiKey: string) => {
      setValidating(true);
      try {
        const isValid = await authService.validateApiKey(apiKey);
        if (!isValid) {
          throw new Error('Invalid API key');
        }
        return apiKey;
      } finally {
        setValidating(false);
      }
    },
    onSuccess: (apiKey) => {
      authenticate(apiKey);
      navigate(ROUTES.DASHBOARD);
      toast({ title: 'Connected', description: 'Successfully authenticated with the server.' });
    },
    onError: () => {
      toast({ title: 'Authentication failed', description: 'Invalid API key. Please check and try again.', variant: 'destructive' });
    },
  });
}

/**
 * Hook to logout (clear API key)
 */
export function useLogout() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const logout = useAuthStore((state) => state.logout);

  return useMutation({
    mutationFn: async () => {
      logout();
      authService.logout();
    },
    onSuccess: () => {
      queryClient.clear();
      navigate(ROUTES.LOGIN);
    },
  });
}

/**
 * Check if authenticated
 */
export function useIsAuthenticated() {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  return isAuthenticated || authService.isAuthenticated();
}

/**
 * Get server version
 */
export function useServerVersion() {
  return useAuthStore((state) => state.serverVersion);
}
