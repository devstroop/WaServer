import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { auth } from '../api/endpoints';
import { clearToken, getToken, setToken } from '../api/client';
import type { UserInfo } from '../api/types';

type AuthState = {
  user: UserInfo | null;
  loading: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  refresh: () => Promise<void>;
};

const Ctx = createContext<AuthState | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserInfo | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = async () => {
    const token = getToken();
    if (!token) {
      setUser(null);
      setLoading(false);
      return;
    }
    try {
      const u = await auth.validate();
      setUser(u);
    } catch {
      clearToken();
      setUser(null);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  const login = async (username: string, password: string) => {
    const res = await auth.login({ username, password });
    setToken(res.token);
    setUser(res.user);
  };

  const logout = async () => {
    try {
      await auth.logout();
    } finally {
      clearToken();
      setUser(null);
    }
  };

  return <Ctx.Provider value={{ user, loading, login, logout, refresh }}>{children}</Ctx.Provider>;
}

export function useAuth() {
  const v = useContext(Ctx);
  if (!v) throw new Error('useAuth outside provider');
  return v;
}
