import { api } from '@/lib/axios';

// Response types from backend
export interface Instance {
  id: string;
  name: string;
  phone_number: string | null;
  status: 'active' | 'inactive' | 'warming_up' | 'error';
  authorized: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateInstanceRequest {
  name: string;
  browser_overrides?: BrowserOverrides;
}

export interface BrowserOverrides {
  headless?: boolean;
  viewport_width?: number;
  viewport_height?: number;
  user_agent?: string;
}

export interface InstanceConfig {
  browser?: InstanceBrowserConfig;
  webhook?: InstanceWebhookConfig;
  rate_limits?: InstanceRateLimits;
}

export interface InstanceBrowserConfig {
  headless: boolean;
  viewport_width: number;
  viewport_height: number;
  user_agent: string | null;
}

export interface InstanceWebhookConfig {
  enabled: boolean;
  endpoints: WebhookEndpoint[];
}

export interface WebhookEndpoint {
  url: string;
  events: string[];
  secret: string | null;
}

export interface InstanceRateLimits {
  messages_per_minute: number;
  messages_per_hour: number;
}

export interface UpdateInstanceConfigRequest {
  browser?: Partial<InstanceBrowserConfig>;
  webhook?: Partial<InstanceWebhookConfig>;
  rate_limits?: Partial<InstanceRateLimits>;
}

export interface InstanceListResponse {
  instances: Instance[];
  total: number;
}

// Base path for instance API
const BASE = '/v1/instances';

export const instanceService = {
  // List all instances
  list: () => api.get<InstanceListResponse>(BASE),

  // Get single instance
  get: (id: string) => api.get<Instance>(`${BASE}/${id}`),

  // Create new instance
  create: (data: CreateInstanceRequest) => api.post<Instance>(BASE, data),

  // Delete instance
  delete: (id: string, deleteSession = false) =>
    api.del<{ success: boolean }>(`${BASE}/${id}?delete_session=${deleteSession}`),

  // Warmup instance browser
  warmup: (id: string) => api.post<{ success: boolean }>(`${BASE}/${id}/warmup`),

  // Reset instance (wipe session data)
  reset: (id: string) => api.del<{ success: boolean }>(`${BASE}/${id}/reset`),

  // Get browser screenshot
  getScreenshot: (id: string) => api.get<{ screenshot: string }>(`${BASE}/${id}/screenshot`),

  // Instance configuration
  getConfig: (id: string) => api.get<InstanceConfig>(`${BASE}/${id}/config`),

  updateConfig: (id: string, config: UpdateInstanceConfigRequest) =>
    api.put<InstanceConfig>(`${BASE}/${id}/config`, config),
};
