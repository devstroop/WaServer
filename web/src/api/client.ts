const BASE = import.meta.env.VITE_API_BASE || '';

function authHeaders(): HeadersInit {
  const token = localStorage.getItem('was_token');
  return token ? { Authorization: `Bearer ${token}` } : {};
}

export class ApiError extends Error {
  status: number;
  correlationId?: string;
  retryAfter?: number;

  constructor(opts: { status: number; message: string; correlationId?: string; retryAfter?: number }) {
    super(opts.message);
    this.name = 'ApiError';
    this.status = opts.status;
    this.message = opts.message;
    this.correlationId = opts.correlationId;
    this.retryAfter = opts.retryAfter;
  }
}

type ErrorEnvelopeDto = {
  error: string;
  message: string;
  correlation_id?: string | null;
  correlationId?: string | null;
};

function parseRetryAfter(value: string | null): number | undefined {
  if (!value) return undefined;
  const n = Number(value.trim());
  return Number.isFinite(n) ? n : undefined;
}

export async function apiFetch<T>(path: string, opts: RequestInit = {}): Promise<T> {
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
    ...authHeaders(),
    ...(opts.headers || {}),
  };
  if (opts.body instanceof FormData) {
    // @ts-expect-error delete content-type
    delete headers['Content-Type'];
  }
  const res = await fetch(`${BASE}${path}`, { ...opts, headers });
  if (!res.ok) {
    const retryAfter = parseRetryAfter(res.headers.get('retry-after') ?? res.headers.get('Retry-After'));
    const headerCorrelationId =
      res.headers.get('x-correlation-id') ?? res.headers.get('X-Correlation-Id') ?? undefined;

    let message = `HTTP ${res.status}`;
    let correlationId: string | undefined = headerCorrelationId ?? undefined;

    try {
      const text = await res.text();
      if (text) {
        try {
          const j = JSON.parse(text) as Partial<ErrorEnvelopeDto> & {
            correlationId?: string | null;
          };
          const bodyCorrelation = j.correlation_id ?? j.correlationId ?? null;
          if (bodyCorrelation) correlationId = bodyCorrelation;
          message = j.message || j.error || text || message;
        } catch {
          message = text || message;
        }
      }
    } catch {
      // ignore body read errors
    }

    throw new ApiError({
      status: res.status,
      message,
      correlationId,
      retryAfter,
    });
  }
  const ct = res.headers.get('content-type') || '';
  if (ct.includes('image')) {
    return (await res.blob()) as unknown as T;
  }
  if (ct.includes('application/json')) {
    return (await res.json()) as T;
  }
  return (await res.text()) as unknown as T;
}

export function setToken(token: string) {
  localStorage.setItem('was_token', token);
}
export function clearToken() {
  localStorage.removeItem('was_token');
}
export function getToken() {
  return localStorage.getItem('was_token');
}
