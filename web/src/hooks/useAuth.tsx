import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import { auth } from '../api/endpoints';
import { ApiError, clearToken, getToken, setToken } from '../api/client';
import type { RegisterRequest, UserInfo } from '../api/types';
import { useToast } from '@devstroop/react-uikit';

type AuthState = {
  user: UserInfo | null;
  loading: boolean;
  isAuthenticated: boolean;
  isAdmin: boolean;
  login: (username: string, password: string) => Promise<void>;
  register: (body: RegisterRequest) => Promise<UserInfo>;
  logout: () => Promise<void>;
  logoutAll: () => Promise<void>;
  refresh: () => Promise<void>;
};

const Ctx = createContext<AuthState | null>(null);

function isAdminRole(role: string | undefined): boolean {
  return (role ?? '').toLowerCase() === 'admin';
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserInfo | null>(null);
  const [loading, setLoading] = useState(true);
  // Toast is optional during early render — App wraps with ToastProvider, but keep safe fallback
  let toastApi: ReturnType<typeof useToast> | null = null;
  try {
    // eslint-disable-next-line react-hooks/rules-of-hooks
    toastApi = useToast();
  } catch {
    toastApi = null;
  }

  const isAuthenticated = useMemo(() => !!user && !!getToken() && user.is_active !== false, [user]);
  const isAdmin = useMemo(() => !!user && isAdminRole(user.role) && user.is_active !== false, [user]);

  const handleInactive = useCallback(
    (correlationId?: string) => {
      clearToken();
      setUser(null);
      if (toastApi) {
        const desc = correlationId
          ? `Account deactivated — contact admin (ref: ${correlationId})`
          : 'Account deactivated — please sign in again.';
        toastApi.toast({
          tone: 'danger',
          title: 'Account inactive',
          description: desc,
          durationMs: 6000,
        });
      }
    },
    [toastApi],
  );

  const refresh = useCallback(async () => {
    const token = getToken();
    if (!token) {
      setUser(null);
      setLoading(false);
      return;
    }
    try {
      const u = await auth.validate();
      if (!u.is_active) {
        handleInactive(undefined);
        return;
      }
      setUser(u);
    } catch (e) {
      const err = e as ApiError;
      const status = (err as unknown as { status?: number })?.status;
      const correlationId = (err as unknown as { correlationId?: string })?.correlationId;
      const msg = (err?.message ?? '').toLowerCase();

      clearToken();
      setUser(null);

      const isInactive = msg.includes('inactive') || msg.includes('user_inactive');
      const isAuthError = status === 401 || status === 403 || isInactive || msg.includes('invalid_token') || msg.includes('invalid or expired');

      if (isInactive) {
        if (toastApi) {
          toastApi.toast({
            tone: 'danger',
            title: 'Account inactive',
            description: correlationId
              ? `Your account has been deactivated (ref: ${correlationId}). Please contact admin.`
              : 'Your account has been deactivated. Please contact admin.',
            durationMs: 6000,
          });
        }
      } else if (isAuthError && toastApi && status === 401) {
        // Only toast for mid-session expiry if we previously had a user — avoid toast on first load without session
        // But also handle generic 401 after token existed: show session expired with correlation id
        // We do not toast for initial no-token case (handled above)
        // For mid-session, correlationId helps debugging
        if (correlationId || msg.includes('expired')) {
          toastApi.toast({
            tone: 'danger',
            title: 'Session expired',
            description: correlationId ? `Please sign in again (ref: ${correlationId})` : 'Please sign in again.',
            durationMs: 5000,
          });
        }
      }
    } finally {
      setLoading(false);
    }
  }, [handleInactive, toastApi]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  // Also handle is_active flipped mid-session via user object (e.g., polling or manual update)
  useEffect(() => {
    if (user && user.is_active === false) {
      handleInactive(undefined);
    }
  }, [user, handleInactive]);

  const login = useCallback(async (username: string, password: string) => {
    const res = await auth.login({ username, password });
    setToken(res.token);
    setUser(res.user);
  }, []);

  const register = useCallback(async (body: RegisterRequest): Promise<UserInfo> => {
    const u = await auth.register(body);
    return u;
  }, []);

  const logout = useCallback(async () => {
    try {
      await auth.logout();
    } catch {
      // ignore network errors, still clear local state
    } finally {
      clearToken();
      setUser(null);
    }
  }, []);

  const logoutAll = useCallback(async () => {
    try {
      await auth.logoutAll();
    } catch {
      // ignore
    } finally {
      clearToken();
      setUser(null);
    }
  }, []);

  const value = useMemo<AuthState>(
    () => ({
      user,
      loading,
      isAuthenticated,
      isAdmin,
      login,
      register,
      logout,
      logoutAll,
      refresh,
    }),
    [user, loading, isAuthenticated, isAdmin, login, register, logout, logoutAll, refresh],
  );

  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useAuth() {
  const v = useContext(Ctx);
  if (!v) throw new Error('useAuth outside provider');
  return v;
}
