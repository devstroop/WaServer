/**
 * Users API Service
 */

import { api } from './client';
import type { User, CreateUserRequest, UpdateUserRequest, InstanceOwner, AccessToken, AccessTokenInfo } from '@/types/user';

// Re-export types for convenience
export type { User, CreateUserRequest, UpdateUserRequest, InstanceOwner, AccessToken };

export const usersApi = {
  /**
   * List all users
   */
  listUsers: async () => {
    const response = await api.get<{ users: User[]; total: number }>('/v1/users');
    return response.users;
  },

  /**
   * Get user by ID
   */
  getUser: (id: string) => api.get<User>(`/v1/users/${id}`),

  /**
   * Get current user info
   */
  getCurrentUser: () => api.get<User>('/v1/users/me'),

  /**
   * Create a new user (admin only)
   */
  createUser: (data: CreateUserRequest) =>
    api.post<{ user: User }>('/v1/users', data),

  /**
   * Update a user
   */
  updateUser: (id: string, data: UpdateUserRequest) =>
    api.patch<User>(`/v1/users/${id}`, data),

  /**
   * Delete a user
   */
  deleteUser: (id: string) => api.delete<{ message: string }>(`/v1/users/${id}`),

  /**
   * List user's access tokens
   */
  listAccessTokens: (userId: string) =>
    api.get<{ tokens: AccessTokenInfo[] }>(`/v1/users/${userId}/tokens`),

  /**
   * Create a new access token
   */
  createAccessToken: (userId: string, name: string, expiresInDays?: number) =>
    api.post<{ token_info: AccessTokenInfo; access_token: string }>(`/v1/users/${userId}/tokens`, {
      name,
      expires_in_days: expiresInDays,
    }),

  /**
   * Delete an access token
   */
  deleteAccessToken: (userId: string, tokenId: string) =>
    api.delete<{ message: string }>(`/v1/users/${userId}/tokens/${tokenId}`),

  /**
   * Get user's instance permissions
   */
  getInstances: (id: string) =>
    api.get<{ instances: InstanceOwner[] }>(`/v1/users/${id}/instances`),

  /**
   * Assign instance to user
   */
  assignInstance: (userId: string, instanceId: string, permission: string) =>
    api.post<InstanceOwner>('/v1/users/assign-instance', {
      user_id: userId,
      instance_id: instanceId,
      permission,
    }),

  /**
   * Remove instance from user
   */
  removeInstance: (userId: string, instanceId: string) =>
    api.delete<{ message: string }>(`/v1/users/${userId}/instances/${instanceId}`),
};
