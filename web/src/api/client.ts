/**
 * API Client for WAS Backend
 * Centralized HTTP client with error handling and auth
 */

import { getApiKey } from '@/store/authStore';

// API Error class
export class ApiError extends Error {
  public readonly status: number;
  public readonly code: string;

  constructor(message: string, status: number, code = 'api_error') {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.code = code;
  }

  static fromResponse(response: Response, data: { error?: string; message?: string }): ApiError {
    return new ApiError(
      data.message || data.error || 'An unexpected error occurred',
      response.status,
      data.error || 'api_error'
    );
  }
}

// Request options
interface RequestOptions {
  method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  body?: unknown;
  headers?: Record<string, string>;
  auth?: boolean;
  apiKey?: string; // Override API key for auth
}

// Base URL for API
const BASE_URL = '/api';

/**
 * Core request function
 */
async function request<T>(endpoint: string, options: RequestOptions): Promise<T> {
  const { method, body, headers = {}, auth = true, apiKey: overrideKey } = options;

  const url = `${BASE_URL}${endpoint}`;

  const requestHeaders: Record<string, string> = {
    'Content-Type': 'application/json',
    ...headers,
  };

  // Add auth header if required
  if (auth) {
    const apiKey = overrideKey || getApiKey();
    if (apiKey) {
      requestHeaders['Authorization'] = `Bearer ${apiKey}`;
    }
  }

  try {
    const response = await fetch(url, {
      method,
      headers: requestHeaders,
      body: body ? JSON.stringify(body) : undefined,
    });

    // Handle no-content responses
    if (response.status === 204) {
      return {} as T;
    }

    const data = await response.json();

    if (!response.ok) {
      throw ApiError.fromResponse(response, data);
    }

    return data as T;
  } catch (error) {
    if (error instanceof ApiError) {
      throw error;
    }
    throw new ApiError(
      error instanceof Error ? error.message : 'Network error',
      0,
      'network_error'
    );
  }
}

// HTTP method helpers
export const api = {
  get: <T>(endpoint: string, options?: { auth?: boolean; apiKey?: string }) =>
    request<T>(endpoint, { method: 'GET', auth: options?.auth ?? true, apiKey: options?.apiKey }),

  post: <T>(endpoint: string, body?: unknown, options?: { auth?: boolean; apiKey?: string }) =>
    request<T>(endpoint, { method: 'POST', body, auth: options?.auth ?? true, apiKey: options?.apiKey }),

  put: <T>(endpoint: string, body?: unknown, options?: { auth?: boolean; apiKey?: string }) =>
    request<T>(endpoint, { method: 'PUT', body, auth: options?.auth ?? true, apiKey: options?.apiKey }),

  patch: <T>(endpoint: string, body?: unknown, options?: { auth?: boolean; apiKey?: string }) =>
    request<T>(endpoint, { method: 'PATCH', body, auth: options?.auth ?? true, apiKey: options?.apiKey }),

  delete: <T>(endpoint: string, options?: { auth?: boolean; apiKey?: string }) =>
    request<T>(endpoint, { method: 'DELETE', auth: options?.auth ?? true, apiKey: options?.apiKey }),
};
