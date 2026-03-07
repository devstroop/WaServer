import axios, { AxiosError, type AxiosRequestConfig, type AxiosResponse } from 'axios';
import { API_BASE_URL, STORAGE_KEYS, ROUTES } from './constants';

export interface ApiErrorResponse {
  message: string;
  code?: string;
  details?: Record<string, string[]>;
}

const apiClient = axios.create({
  baseURL: API_BASE_URL,
  timeout: 30000,
  headers: { 'Content-Type': 'application/json' },
});

// Request interceptor - add API key as Bearer token
apiClient.interceptors.request.use(
  (config) => {
    const apiKey = localStorage.getItem(STORAGE_KEYS.AUTH_TOKEN);
    if (apiKey) {
      config.headers.Authorization = `Bearer ${apiKey}`;
    }
    return config;
  },
  (error) => Promise.reject(error)
);

// Response interceptor - handle errors
apiClient.interceptors.response.use(
  (response) => response,
  async (error: AxiosError<ApiErrorResponse>) => {
    if (error.response?.status === 401) {
      // API key is invalid or missing - redirect to login
      const currentPath = window.location.pathname;
      if (currentPath !== ROUTES.LOGIN) {
        window.location.href = `${ROUTES.LOGIN}?error=unauthorized`;
      }
    }

    if (error.response?.status === 503) {
      // Service unavailable - retry after delay
      const config = error.config;
      if (config && !config.headers['X-Retry']) {
        config.headers['X-Retry'] = 'true';
        await new Promise((resolve) => setTimeout(resolve, 2000));
        return apiClient(config);
      }
    }

    return Promise.reject(error);
  }
);

// Typed request helpers
export async function get<T>(url: string, config?: AxiosRequestConfig): Promise<T> {
  const response: AxiosResponse<T> = await apiClient.get(url, config);
  return response.data;
}

export async function post<T, D = unknown>(url: string, data?: D, config?: AxiosRequestConfig): Promise<T> {
  const response: AxiosResponse<T> = await apiClient.post(url, data, config);
  return response.data;
}

export async function put<T, D = unknown>(url: string, data?: D, config?: AxiosRequestConfig): Promise<T> {
  const response: AxiosResponse<T> = await apiClient.put(url, data, config);
  return response.data;
}

export async function del<T>(url: string, config?: AxiosRequestConfig): Promise<T> {
  const response: AxiosResponse<T> = await apiClient.delete(url, config);
  return response.data;
}

export async function upload<T>(url: string, formData: FormData): Promise<T> {
  const response: AxiosResponse<T> = await apiClient.post(url, formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
  });
  return response.data;
}

// Convenience object for all API methods
export const api = {
  get,
  post,
  put,
  del,
  upload,
};

export { apiClient };
