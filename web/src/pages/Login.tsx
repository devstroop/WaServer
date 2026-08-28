import { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';

export default function Login() {
  const { login } = useAuth();
  const nav = useNavigate();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [err, setErr] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr(null);
    setLoading(true);
    try {
      await login(username, password);
      nav('/');
    } catch (e: unknown) {
      setErr(e instanceof Error ? e.message : 'Login failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="mx-auto mt-16 max-w-sm rounded-lg border bg-white p-6 shadow-sm">
      <h1 className="mb-1 text-xl font-semibold">Sign in — WAS</h1>
      <p className="mb-4 text-sm text-zinc-500">Backend: {import.meta.env.VITE_API_BASE || 'proxy → http://localhost:3000'}</p>
      <form onSubmit={onSubmit} className="space-y-3">
        <input
          className="w-full rounded border px-3 py-2 text-sm"
          placeholder="Username or email"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          required
        />
        <input
          className="w-full rounded border px-3 py-2 text-sm"
          placeholder="Password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          required
        />
        {err && <div className="rounded bg-red-50 p-2 text-sm text-red-700">{err}</div>}
        <button
          disabled={loading}
          className="w-full rounded bg-violet-600 px-3 py-2 text-sm font-medium text-white hover:bg-violet-700 disabled:opacity-50"
        >
          {loading ? 'Signing in…' : 'Sign in'}
        </button>
      </form>
      <p className="mt-4 text-center text-xs text-zinc-500">
        Need account? <Link to="/register" className="text-violet-600">Register</Link>
      </p>
    </div>
  );
}
