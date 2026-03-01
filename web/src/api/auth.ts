/**
 * Authentication API Service
 */

import { api } from './client';
import type { User } from '@/types/user';

// Auth types
export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  user: User;
  token: string;
  expires_at: string;
}

export interface RegisterRequest {
  username: string;
  email?: string;
  password: string;
}

export const authApi = {
  /**
   * Login with username/email and password
   */
  login: (data: LoginRequest) =>
    api.post<LoginResponse>('/v1/auth/login', data, { auth: false }),

  /**
   * Register a new user account
   */
  register: (data: RegisterRequest) =>
    api.post<User>('/v1/auth/register', data, { auth: false }),

  /**
   * Validate current session token
   */
  validate: () => api.get<User>('/v1/auth/validate'),

  /**
   * Logout (invalidate session token)
   */
  logout: () => api.post<{ message: string }>('/v1/auth/logout'),

  /**
   * Check if a token is valid (for app initialization)
   */
  validateToken: (token: string) =>
    api.get<User>('/v1/auth/validate', { apiKey: token }),
};
