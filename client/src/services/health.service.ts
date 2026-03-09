import { api } from '@/lib/axios';

// Health endpoints are at /api (no /v1 prefix, no auth)

export interface HealthResponse {
  status: 'healthy' | 'degraded' | 'unhealthy';
  version: string;
  uptime_seconds: number;
  services: ServiceHealth[];
}

export interface ServiceHealth {
  name: string;
  status: 'healthy' | 'unhealthy';
  message: string | null;
}

export interface StatusResponse {
  status: 'ok' | 'error';
}

export interface MetricsResponse {
  uptime_seconds: number;
  total_requests: number;
  active_instances: number;
  messages_sent: number;
  messages_received: number;
}

export const healthService = {
  // Detailed health check
  check: () => api.get<HealthResponse>('/health'),

  // Kubernetes readiness probe
  ready: () => api.get<StatusResponse>('/ready'),

  // Kubernetes liveness probe
  live: () => api.get<StatusResponse>('/live'),

  // Server metrics
  metrics: () => api.get<MetricsResponse>('/metrics'),
};
