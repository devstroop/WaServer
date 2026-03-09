import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { STORAGE_KEYS } from '@/lib/constants';

/**
 * Auth state for API key authentication.
 * The backend uses a simple secret key for auth - no user accounts.
 */
interface AuthState {
  apiKey: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  isValidating: boolean;
  serverVersion: string | null;

  // Actions
  setApiKey: (apiKey: string | null) => void;
  authenticate: (apiKey: string) => void;
  logout: () => void;
  setLoading: (loading: boolean) => void;
  setValidating: (validating: boolean) => void;
  setServerVersion: (version: string | null) => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      apiKey: null,
      isAuthenticated: false,
      isLoading: true,
      isValidating: false,
      serverVersion: null,

      setApiKey: (apiKey) => {
        if (apiKey) {
          localStorage.setItem(STORAGE_KEYS.AUTH_TOKEN, apiKey);
        } else {
          localStorage.removeItem(STORAGE_KEYS.AUTH_TOKEN);
        }
        set({ apiKey, isAuthenticated: !!apiKey });
      },

      authenticate: (apiKey) => {
        localStorage.setItem(STORAGE_KEYS.AUTH_TOKEN, apiKey);
        set({ apiKey, isAuthenticated: true, isLoading: false });
      },

      logout: () => {
        localStorage.removeItem(STORAGE_KEYS.AUTH_TOKEN);
        set({ apiKey: null, isAuthenticated: false });
      },

      setLoading: (isLoading) => set({ isLoading }),
      setValidating: (isValidating) => set({ isValidating }),
      setServerVersion: (serverVersion) => set({ serverVersion }),
    }),
    {
      name: 'was-auth-storage',
      partialize: (state) => ({ apiKey: state.apiKey }),
    }
  )
);
