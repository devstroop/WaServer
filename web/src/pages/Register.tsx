import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { auth } from '../api/endpoints';

export default function Register() {
  const nav = useNavigate();
  const [username, setUsername] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [err, setErr] = useState<string | null>(null);
  const [ok, setOk] = useState<string | null>(null);

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr(null);
    setOk(null);
    try {
      await auth.register({ username, email: email || undefined, password });
      setOk('Registered. First user becomes admin. Now sign in.');
      setTimeout(() => nav('/login'), 1200);
    } catch (e: unknown) {
      setErr(e instanceof Error ? e.message : 'Register failed');
    }
  };

  return (
    <div className="mx-auto mt-16 max-w-sm rounded-lg border bg-white p-6 shadow-sm">
      <h1 className="mb-4 text-xl font-semibold">Create account</h1>
      <form onSubmit={onSubmit} className="space-y-3">
        <input className="w-full rounded border px-3 py-2 text-sm" placeholder="Username" value={username} onChange={(e) => setUsername(e.target.value)} required />
        <input className="w-full rounded border px-3 py-2 text-sm" placeholder="Email (optional)" value={email} onChange={(e) => setEmail(e.target.value)} />
        <input className="w-full rounded border px-3 py-2 text-sm" placeholder="Password (≥8)" type="password" value={password} onChange={(e) => setPassword(e.target.value)} required />
        {err && <div className="rounded bg-red-50 p-2 text-sm text-red-700">{err}</div>}
        {ok && <div className="rounded bg-green-50 p-2 text-sm text-green-700">{ok}</div>}
        <button className="w-full rounded bg-violet-600 px-3 py-2 text-sm font-medium text-white hover:bg-violet-700">Create</button>
      </form>
      <p className="mt-4 text-center text-xs text-zinc-500">
        Have account? <Link to="/login" className="text-violet-600">Sign in</Link>
      </p>
    </div>
  );
}
