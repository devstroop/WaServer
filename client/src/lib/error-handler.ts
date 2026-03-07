import { AxiosError } from 'axios';
import type { ApiErrorResponse } from './axios';

export interface ApiError {
  message: string;
  code?: string | undefined;
  status?: number | undefined;
  details?: Record<string, string[]> | undefined;
}

export function handleApiError(error: unknown): ApiError {
  if (error instanceof AxiosError) {
    const response = error.response?.data as ApiErrorResponse | undefined;
    return {
      message: response?.message || error.message || 'An unexpected error occurred',
      code: response?.code ?? error.code,
      status: error.response?.status,
      details: response?.details,
    };
  }
  if (error instanceof Error) {
    return { message: error.message };
  }
  return { message: 'An unexpected error occurred' };
}

export function isApiError(error: unknown): error is ApiError {
  return typeof error === 'object' && error !== null && 'message' in error;
}

export function getFieldError(error: ApiError | undefined, field: string): string | undefined {
  return error?.details?.[field]?.[0];
}
