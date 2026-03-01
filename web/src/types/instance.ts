/**
 * Instance Types
 */

export type InstanceStatus = 'sleeping' | 'starting' | 'ready' | 'connected' | 'disconnected' | 'error';

export interface Instance {
  id: string;
  phone_number?: string;
  instance_name?: string;
  status: InstanceStatus;
  authorized?: boolean;
  created_at?: string;
}

export interface CreateInstanceRequest {
  phone_number: string;
  instance_name?: string;
  idle_timeout?: number;
}

export interface InstanceConfig {
  browser?: {
    headless?: boolean;
    timeout_ms?: number;
  };
  webhooks?: {
    enabled?: boolean;
    endpoints?: Array<{
      url: string;
      events: string[];
    }>;
  };
  rate_limits?: {
    messages_per_minute?: number;
    messages_per_hour?: number;
  };
}

export interface InstanceProfile {
  name?: string;
  status?: string;
  push_name?: string;
}
