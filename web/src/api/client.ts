const BASE = import.meta.env.VITE_API_BASE || '';

function authHeaders(): HeadersInit {
  const token = localStorage.getItem('was_token');
  return token ? { Authorization: `Bearer ${token}` } : {};
}

export async function apiFetch<T>(path: string, opts: RequestInit = {}): Promise<T> {
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
    ...authHeaders(),
    ...(opts.headers || {}),
  };
  // Don't set Content-Type for FormData
  if (opts.body instanceof FormData) {
    // @ts-expect-error delete content-type
    delete headers['Content-Type'];
  }
  const res = await fetch(`${BASE}${path}`, { ...opts, headers });
  if (!res.ok) {
    const text = await res.text();
    let message = text;
    try {
      const j = JSON.parse(text);
      message = j.message || j.error || text;
    } catch {}
    throw new Error(message || `HTTP ${res.status}`);
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
