/**
 * Auth Store with Zustand
 * Manages authentication state and session token storage
 */

import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';

export type UserRole = 'admin' | 'user';

export interface AuthUser {
  id: string;
  username: string;
  email?: string;
  role: UserRole;
  is_active: boolean;
}

interface AuthState {
  // Persisted state
  user: AuthUser | null;
  token: string | null;
  isAuthenticated: boolean;
  isSuperadmin: boolean;

  // Actions
  login: (user: AuthUser, token: string) => void;
  loginAsSuperadmin: (secretKey: string) => void;
  logout: () => void;
  setUser: (user: AuthUser | null) => void;
  setAuth: (token: string | null, user: AuthUser | null) => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      token: null,
      isAuthenticated: false,
      isSuperadmin: false,

      login: (user, token) => {
        set({
          user,
          token,
          isAuthenticated: true,
          isSuperadmin: false,
        });
      },

      loginAsSuperadmin: (secretKey) => {
        set({
          user: {
            id: 'superadmin',
            username: 'superadmin',
            role: 'admin',
            is_active: true,
          },
          token: secretKey,
          isAuthenticated: true,
          isSuperadmin: true,
        });
      },

      logout: () => {
        set({
          user: null,
          token: null,
          isAuthenticated: false,
          isSuperadmin: false,
        });
      },

      setUser: (user) => set({ user }),

      setAuth: (token, user) => {
        if (token) {
          set({
            token,
            user,
            isAuthenticated: true,
          });
        } else {
          set({
            token: null,
            user: null,
            isAuthenticated: false,
          });
        }
      },
    }),
    {
      name: 'was-auth',
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        user: state.user,
        token: state.token,
        isAuthenticated: state.isAuthenticated,
        isSuperadmin: state.isSuperadmin,
      }),
    }
  )
);

/**
 * Get current token for requests (renamed from getApiKey for compatibility)
 */
export function getApiKey(): string | null {
  return useAuthStore.getState().token;
}

/**
 * Get current auth token
 */
export function getToken(): string | null {
  return useAuthStore.getState().token;
}

/**
 * Check if current user is admin (superadmin or admin role)
 */
export function isAdmin(): boolean {
  const state = useAuthStore.getState();
  return state.isSuperadmin || state.user?.role === 'admin';
}
