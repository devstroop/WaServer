/**
 * User Types
 */

export type UserRole = 'admin' | 'user';
export type InstancePermission = 'owner' | 'operator' | 'viewer';

export interface User {
  id: string;
  username: string;
  email?: string;
  role: UserRole;
  is_active: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface CreateUserRequest {
  username: string;
  email?: string;
  password: string;
  role?: UserRole;
}

export interface UpdateUserRequest {
  username?: string;
  email?: string;
  password?: string;
  role?: UserRole;
  is_active?: boolean;
}

export interface InstanceOwner {
  user_id: string;
  instance_id: string;
  permission: InstancePermission;
  created_at?: string;
}

export interface AccessToken {
  id: string;
  user_id: string;
  name: string;
  token_hash: string;
  expires_at?: string;
  last_used?: string;
  created_at?: string;
}

export interface AccessTokenInfo {
  id: string;
  name: string;
  expires_at?: string;
  last_used?: string;
  created_at?: string;
}
