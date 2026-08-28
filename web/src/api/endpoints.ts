import { apiFetch } from './client';
import type {
  HealthResponse,
  InstanceListResponse,
  CreateInstanceRequest,
  CreateInstanceResponse,
  UserInfo,
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  AccessTokenInfo,
  SendMessageResponse,
  WhatsAppStatus,
  InstanceInfo,
  InstanceOwnerRecord,
  UserInstancesResponse,
  AssignInstanceRequest,
} from './types';

export const health = {
  get: () => apiFetch<HealthResponse>('/api/health'),
  ready: () => apiFetch<{ status: string }>('/api/ready'),
  live: () => apiFetch<{ status: string }>('/api/live'),
  metrics: () => apiFetch<unknown>('/api/metrics'),
};

export const auth = {
  login: (body: LoginRequest) =>
    apiFetch<LoginResponse>('/api/v1/auth/login', { method: 'POST', body: JSON.stringify(body) }),
  register: (body: RegisterRequest) =>
    apiFetch<UserInfo>('/api/v1/auth/register', { method: 'POST', body: JSON.stringify(body) }),
  validate: () => apiFetch<UserInfo>('/api/v1/auth/validate'),
  logout: () => apiFetch<{ message: string }>('/api/v1/auth/logout', { method: 'POST' }),
  logoutAll: () => apiFetch<{ message: string; revoked: number }>('/api/v1/auth/logout-all', { method: 'POST' }),
};

export const instances = {
  list: () => apiFetch<InstanceListResponse>('/api/v1/instances'),
  create: (body: CreateInstanceRequest) =>
    apiFetch<CreateInstanceResponse>('/api/v1/instances', { method: 'POST', body: JSON.stringify(body) }),
  get: (id: string) => apiFetch<InstanceInfo>(`/api/v1/instances/${id}`),
  del: (id: string, deleteData = false) =>
    apiFetch<{ message: string }>(`/api/v1/instances/${id}?delete_data=${deleteData}`, { method: 'DELETE' }),
  warmup: (id: string) => apiFetch<{ message: string }>(`/api/v1/instances/${id}/warmup`, { method: 'POST' }),
  screenshot: (id: string) => apiFetch<Blob>(`/api/v1/instances/${id}/screenshot`),
  getConfig: (id: string) => apiFetch<unknown>(`/api/v1/instances/${id}/config`),
  updateConfig: (id: string, body: unknown) =>
    apiFetch<unknown>(`/api/v1/instances/${id}/config`, { method: 'PUT', body: JSON.stringify(body) }),
  reset: (id: string) => apiFetch<{ message: string }>(`/api/v1/instances/${id}/reset`, { method: 'DELETE' }),
  status: (id: string) => apiFetch<WhatsAppStatus>(`/api/v1/instances/${id}/status`),
  qr: (id: string) => apiFetch<Blob>(`/api/v1/instances/${id}/link/qr`),
  linkPhone: (id: string) => apiFetch<{ linking_code: string }>(`/api/v1/instances/${id}/link/phone`, { method: 'POST' }),
  unlink: (id: string) => apiFetch<{ message: string }>(`/api/v1/instances/${id}/unlink`, { method: 'DELETE' }),
  send: (id: string, params: { phone: string; text?: string }, file?: File) => {
    const qs = new URLSearchParams({ phone: params.phone, ...(params.text ? { text: params.text } : {}) });
    if (file) {
      const fd = new FormData();
      fd.append('file', file);
      return apiFetch<SendMessageResponse>(`/api/v1/instances/${id}/send?${qs.toString()}`, {
        method: 'POST',
        body: fd,
      });
    }
    return apiFetch<SendMessageResponse>(`/api/v1/instances/${id}/send?${qs.toString()}`, { method: 'POST' });
  },
};

export const users = {
  list: () => apiFetch<{ users: UserInfo[]; total: number }>('/api/v1/users'),
  create: (body: { username: string; email?: string; password: string; role?: string }) =>
    apiFetch<UserInfo>('/api/v1/users', { method: 'POST', body: JSON.stringify(body) }),
  get: (id: string) => apiFetch<UserInfo>(`/api/v1/users/${id}`),
  update: (id: string, body: Partial<{ username: string; email: string; password: string; role: string; is_active: boolean }>) =>
    apiFetch<UserInfo>(`/api/v1/users/${id}`, { method: 'PATCH', body: JSON.stringify(body) }),
  del: (id: string) => apiFetch<{ message: string }>(`/api/v1/users/${id}`, { method: 'DELETE' }),
  me: () => apiFetch<UserInfo>('/api/v1/users/me'),
  tokens: (userId: string) => apiFetch<{ tokens: AccessTokenInfo[] }>(`/api/v1/users/${userId}/tokens`),
  createToken: (userId: string, body: { name: string }) =>
    apiFetch<{ token_info: AccessTokenInfo; access_token: string }>(`/api/v1/users/${userId}/tokens`, {
      method: 'POST',
      body: JSON.stringify(body),
    }),
  deleteToken: (userId: string, tokenId: string) =>
    apiFetch<{ message: string }>(`/api/v1/users/${userId}/tokens/${tokenId}`, { method: 'DELETE' }),
  instances: (userId: string) => apiFetch<UserInstancesResponse>(`/api/v1/users/${userId}/instances`),
  assign: (body: AssignInstanceRequest) =>
    apiFetch<InstanceOwnerRecord>('/api/v1/users/assign-instance', { method: 'POST', body: JSON.stringify(body) }),
  removeInstance: (userId: string, instanceId: string) =>
    apiFetch<{ message: string }>(`/api/v1/users/${userId}/instances/${instanceId}`, { method: 'DELETE' }),
};
