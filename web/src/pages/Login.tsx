import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { Alert, Button, Card, Checkbox, Field, Input, Password } from '@devstroop/react-uikit';
import { useAuth } from '../hooks/useAuth';

export default function Login() {
  const nav = useNavigate();
  const { login } = useAuth();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [err, setErr] = useState<string | null>(null);
  const [fieldErrors, setFieldErrors] = useState<{ username?: string; password?: string }>({});
  const [loading, setLoading] = useState(false);
  const [remember, setRemember] = useState(true);

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr(null);

    const nextErrors: { username?: string; password?: string } = {};
    if (!username.trim()) nextErrors.username = 'Username is required';
    if (!password) nextErrors.password = 'Password is required';
    if (nextErrors.username || nextErrors.password) {
      setFieldErrors(nextErrors);
      return;
    }
    setFieldErrors({});

    setLoading(true);
    try {
      const user = await login(username.trim(), password);
      const role = (user.role ?? '').toLowerCase();
      if (role === 'admin') nav('/admin/dashboard');
      else nav('/dashboard');
    } catch (e: unknown) {
      let status: number | undefined;
      let retryAfter: number | undefined;
      let message = e instanceof Error ? e.message : 'Login failed';
      if (e && typeof e === 'object') {
        const maybe = e as Record<string, unknown>;
        if (typeof maybe.status === 'number') status = maybe.status;
        if (typeof maybe.retryAfter === 'number') retryAfter = maybe.retryAfter;
        if (typeof maybe.retry_after === 'number') retryAfter = maybe.retry_after as number;
      }

      const lower = message.toLowerCase();

      if (status === 429 || lower.includes('rate_limited') || lower.includes('too many')) {
        if (retryAfter !== undefined) {
          setErr(`Too many attempts — retry in ${retryAfter}s`);
        } else {
          const m = message.match(/retry in (\d+)s/i);
          if (m) setErr(`Too many attempts — retry in ${m[1]}s`);
          else if (lower.includes('too many') || lower.includes('rate_limited')) setErr(message);
          else setErr('Too many failed attempts. Please try again later.');
        }
        return;
      }

      if (status === 401 || lower.includes('invalid_credentials') || lower.includes('invalid username')) {
        if (lower.includes('invalid')) setErr(message);
        else setErr('Invalid username/email or password');
        return;
      }

      setErr(message || 'Login failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="mx-auto mt-12 max-w-sm">
      <div className="mb-5 text-center">
        <Link to="/" className="text-xl font-bold tracking-tight">
          WhatsApp Server
        </Link>
        <p className="mt-1 text-xs text-zinc-500">v0.1.0</p>
      </div>
      <Card header="Sign in" className="shadow-sm">
        <div className="space-y-4">
          <p className="text-sm text-zinc-500">Enter your credentials to continue</p>
          {err && (
            <Alert tone="danger" variant="soft">
              {err}
            </Alert>
          )}
          <form onSubmit={onSubmit} className="space-y-3" noValidate>
            <Field label="Username or email" htmlFor="login-username" required error={fieldErrors.username}>
              <Input
                id="login-username"
                placeholder="Username or email"
                value={username}
                onChange={(ev: React.ChangeEvent<HTMLInputElement>) => {
                  setUsername(ev.target.value);
                  if (fieldErrors.username) setFieldErrors((prev) => ({ ...prev, username: undefined }));
                }}
                invalid={!!fieldErrors.username}
                autoComplete="username"
              />
            </Field>
            <Field label="Password" htmlFor="login-password" required error={fieldErrors.password}>
              <Password
                id="login-password"
                placeholder="Password"
                value={password}
                onChange={(ev: React.ChangeEvent<HTMLInputElement>) => {
                  setPassword(ev.target.value);
                  if (fieldErrors.password) setFieldErrors((prev) => ({ ...prev, password: undefined }));
                }}
                invalid={!!fieldErrors.password}
                autoComplete="current-password"
              />
            </Field>
            <label className="flex items-center gap-2 text-xs text-zinc-600">
              <Checkbox checked={remember} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setRemember(e.target.checked)} />
              Remember me
            </label>
            <Button type="submit" variant="primary" fullWidth disabled={loading} className="mt-1">
              {loading ? 'Signing in…' : 'Sign in'}
            </Button>
          </form>
          <p className="text-center text-xs text-zinc-500">
            Need account?{' '}
            <Link to="/auth/register" className="font-semibold text-primary">
              Register
            </Link>
          </p>
        </div>
      </Card>
    </div>
  );
}
