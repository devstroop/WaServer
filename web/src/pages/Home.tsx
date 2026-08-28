import { Link } from 'react-router-dom';

export default function Home() {
  return (
    <div className="mx-auto max-w-2xl py-16 text-center">
      <h1 className="text-3xl font-bold">WAS — WhatsApp Server</h1>
      <p className="mt-2 text-zinc-600">Backend-only API. React admin is lean by design.</p>
      <div className="mt-6 flex justify-center gap-3">
        <Link to="/login" className="rounded bg-zinc-900 px-4 py-2 text-sm text-white">Sign in</Link>
        <Link to="/register" className="rounded border px-4 py-2 text-sm">Create account</Link>
        <a href="/api-docs/" target="_blank" className="rounded border px-4 py-2 text-sm">API Docs</a>
      </div>
    </div>
  );
}
