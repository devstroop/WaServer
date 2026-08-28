import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { Alert, Button, Card, Field, Input, Password, email as emailValidator, minLength, pattern, required } from '@devstroop/react-uikit';
import { auth } from '../api/endpoints';

export default function Register() {
  const nav = useNavigate();
  const [username, setUsername] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [err, setErr] = useState<string | null>(null);
  const [ok, setOk] = useState<string | null>(null);
  const [fieldErrors, setFieldErrors] = useState<{ username?: string; email?: string; password?: string }>({});
  const [loading, setLoading] = useState(false);

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr(null);
    setOk(null);

    const nextErrors: { username?: string; email?: string; password?: string } = {};
    const usernameError = required('Username is required')(username.trim());
    if (usernameError) nextErrors.username = usernameError;

    const trimmedEmail = email.trim();
    if (trimmedEmail) {
      const emailError =
        emailValidator('Invalid email')(trimmedEmail) ??
        pattern(/^[^\s@]+@[^\s@]+\.[^\s@]+$/, 'Invalid email')(trimmedEmail);
      if (emailError) nextErrors.email = emailError;
    }

    const pwRequired = required('Password is required')(password);
    if (pwRequired) {
      nextErrors.password = pwRequired;
    } else {
      const pwLengthError = minLength(8, 'Password must be at least 8 characters')(password);
      if (pwLengthError) nextErrors.password = pwLengthError;
    }

    if (nextErrors.username || nextErrors.email || nextErrors.password) {
      setFieldErrors(nextErrors);
      return;
    }
    setFieldErrors({});

    setLoading(true);
    try {
      await auth.register({ username: username.trim(), email: trimmedEmail || undefined, password });
      setOk('Registered. First user becomes Admin. Now sign in.');
      setTimeout(() => nav('/login'), 1200);
    } catch (e: unknown) {
      let status: number | undefined;
      let message = e instanceof Error ? e.message : 'Register failed';
      if (e && typeof e === 'object') {
        const maybe = e as Record<string, unknown>;
        if (typeof maybe.status === 'number') status = maybe.status;
      }
      const lower = message.toLowerCase();

      // 409 — duplicate username/email
      if (status === 409 || lower.includes('already exists') || lower.includes('conflict') || lower.includes('duplicate') || message.includes('409')) {
        if (lower.includes('email')) {
          setFieldErrors((prev) => ({ ...prev, email: 'Email already exists' }));
        } else if (lower.includes('username')) {
          setFieldErrors((prev) => ({ ...prev, username: 'Username already exists' }));
        }
        setErr(message.includes('already exists') ? message : 'Username or email already exists');
        return;
      }

      // 400 — validation errors
      if (status === 400 || lower.includes('invalid') || lower.includes('password must') || lower.includes('username cannot') || message.includes('400')) {
        if (lower.includes('password')) {
          setFieldErrors((prev) => ({ ...prev, password: message }));
        } else if (lower.includes('email')) {
          setFieldErrors((prev) => ({ ...prev, email: message }));
        } else if (lower.includes('username')) {
          setFieldErrors((prev) => ({ ...prev, username: message }));
        } else {
          setErr(message);
        }
        return;
      }

      setErr(message || 'Register failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="mx-auto mt-16 max-w-sm">
      <Card header="Create account">
        <div className="space-y-4">
          <Alert tone="info" variant="soft">First user becomes Admin — you will get administrator privileges.</Alert>
          {err && (
            <Alert tone="danger" variant="soft">
              {err}
            </Alert>
          )}
          {ok && (
            <Alert tone="success" variant="soft">
              {ok}
            </Alert>
          )}
          <form onSubmit={onSubmit} className="space-y-3" noValidate>
            <Field label="Username" htmlFor="register-username" required error={fieldErrors.username}>
              <Input
                id="register-username"
                placeholder="Username"
                value={username}
                onChange={(ev: React.ChangeEvent<HTMLInputElement>) => {
                  setUsername(ev.target.value);
                  if (fieldErrors.username) setFieldErrors((prev) => ({ ...prev, username: undefined }));
                }}
                invalid={!!fieldErrors.username}
                autoComplete="username"
              />
            </Field>
            <Field label="Email" htmlFor="register-email" hint="Optional" error={fieldErrors.email}>
              <Input
                id="register-email"
                type="email"
                placeholder="Email (optional)"
                value={email}
                onChange={(ev: React.ChangeEvent<HTMLInputElement>) => {
                  setEmail(ev.target.value);
                  if (fieldErrors.email) setFieldErrors((prev) => ({ ...prev, email: undefined }));
                }}
                invalid={!!fieldErrors.email}
                autoComplete="email"
              />
            </Field>
            <Field label="Password" htmlFor="register-password" required error={fieldErrors.password}>
              <Password
                id="register-password"
                placeholder="Password (≥8)"
                value={password}
                onChange={(ev: React.ChangeEvent<HTMLInputElement>) => {
                  setPassword(ev.target.value);
                  if (fieldErrors.password) setFieldErrors((prev) => ({ ...prev, password: undefined }));
                }}
                invalid={!!fieldErrors.password}
                autoComplete="new-password"
              />
            </Field>
            <Button type="submit" variant="primary" fullWidth disabled={loading}>
              {loading ? 'Creating…' : 'Create'}
            </Button>
          </form>
          <p className="text-center text-xs text-zinc-500">
            Have account?{' '}
            <Link to="/login" className="text-violet-600">
              Sign in
            </Link>
          </p>
        </div>
      </Card>
    </div>
  );
}
