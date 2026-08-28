// Mirrors src/interfaces/http/dto/*.rs

export interface HealthResponse {
  status: string;
  timestamp: number;
  version: string;
  uptime_seconds: number;
  instances_count: number;
  browser_available?: boolean;
}

export interface InstanceMetrics {
  id: string;
  status: string;
  authorized: boolean;
  total_messages_sent: number;
  error_count: number;
  warmups?: number | null;
}

export interface MetricsResponse {
  timestamp: number;
  uptime_seconds: number;
  memory_usage_bytes: number;
  instances_count: number;
  instances: InstanceMetrics[];
}

export interface InstanceInfo {
  id: string;
  name: string;
  phone_number: string | null;
  status: string;
  authorized: boolean;
  created_at: string;
  updated_at: string;
}

export interface InstanceListResponse {
  instances: InstanceInfo[];
  total: number;
}

export interface CreateInstanceRequest {
  name: string;
  phone_number?: string;
}

export interface CreateInstanceResponse {
  instance_id: string;
  name: string;
  message: string;
}

export interface UserInfo {
  id: string;
  username: string;
  email?: string | null;
  role: string;
  is_active: boolean;
  created_at: string;
  updated_at?: string | null;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  user: UserInfo;
  token: string;
  expires_at: string;
}

export interface RegisterRequest {
  username: string;
  email?: string;
  password: string;
}

export interface AccessTokenInfo {
  id: string;
  user_id: string;
  name: string;
  expires_at: string | null;
  last_used: string | null;
  created_at: string;
}

export interface SendMessageResponse {
  status: string;
  success: boolean;
  message_id: string;
  phone: string;
  timestamp: string;
}

export interface WhatsAppStatus {
  instance_id: string;
  status: string;
  authorized: boolean;
  phone_number?: string | null;
}
