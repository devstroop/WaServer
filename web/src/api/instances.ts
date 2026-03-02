/**
 * Instances API Service
 */

import { api } from './client';
import type { Instance, CreateInstanceRequest, InstanceConfig } from '@/types/instance';

// Re-export types for convenience
export type { Instance, CreateInstanceRequest, InstanceConfig };

export interface LinkingSession {
  qr_code?: string;
  status: string;
}

export const instancesApi = {
  /**
   * List all instances
   */
  listInstances: async () => {
    const response = await api.get<{ instances: Instance[] }>('/v1/instances');
    return response.instances;
  },

  /**
   * Get instance by ID
   */
  getInstance: (id: string) => api.get<Instance>(`/v1/instances/${id}`),

  /**
   * Create a new instance
   */
  createInstance: (instanceId: string) =>
    api.post<{ instance_id: string; data_dir: string }>('/v1/instances', { instance_id: instanceId }),

  /**
   * Delete an instance
   */
  deleteInstance: (id: string, deleteData = false) =>
    api.delete<{ message: string; data_deleted: boolean }>(
      `/v1/instances/${id}?delete_data=${deleteData}`
    ),

  /**
   * Link instance (get QR code)
   */
  linkInstance: (id: string): Promise<LinkingSession> =>
    api.post<LinkingSession>(`/v1/instances/${id}/link`),

  /**
   * Disconnect an instance
   */
  disconnectInstance: (id: string) =>
    api.post<{ message: string }>(`/v1/instances/${id}/disconnect`),

  /**
   * Warmup (start) an instance
   */
  warmup: (id: string) => api.post<{ message: string }>(`/v1/instances/${id}/warmup`),

  /**
   * Reset an instance
   */
  reset: (id: string) => api.delete<{ message: string }>(`/v1/instances/${id}/reset`),

  /**
   * Get instance status
   */
  getStatus: (id: string) =>
    api.get<{
      authenticated: boolean;
      status: string;
      phone_number?: string;
    }>(`/v1/instances/${id}/status`),

  /**
   * Get QR code for linking
   */
  getQrCode: (id: string) => api.get<{ qrcode: string }>(`/v1/instances/${id}/link/qr`),

  /**
   * Link via phone number
   */
  linkPhone: (id: string, phone: string) =>
    api.post<{ code?: string }>(`/v1/instances/${id}/link/phone`, { phone }),

  /**
   * Unlink instance
   */
  unlink: (id: string) => api.delete<{ message: string }>(`/v1/instances/${id}/unlink`),

  /**
   * Get instance config
   */
  getConfig: (id: string) => api.get<InstanceConfig>(`/v1/instances/${id}/config`),

  /**
   * Update instance config
   */
  updateConfig: (id: string, config: Partial<InstanceConfig>) =>
    api.put<InstanceConfig>(`/v1/instances/${id}/config`, config),

  /**
   * Get screenshot
   */
  getScreenshot: (id: string) => api.get<{ screenshot: string }>(`/v1/instances/${id}/screenshot`),
};
