import { Link, Outlet, useNavigate } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';

export default function Layout() {
  const { user, logout } = useAuth();
  const nav = useNavigate();
  const onLogout = async () => {
    await logout();
    nav('/login');
  };
  return (
    <div className="min-h-screen bg-zinc-50 text-zinc-900">
      <header className="sticky top-0 z-10 border-b bg-white">
        <div className="mx-auto flex max-w-6xl items-center justify-between px-4 py-3">
          <Link to="/" className="font-semibold tracking-tight">
            WAS <span className="text-violet-600">Admin</span>
          </Link>
          <nav className="flex items-center gap-4 text-sm">
            <Link to="/app" className="hover:text-violet-600">
              Dashboard
            </Link>
            <Link to="/app/instances" className="hover:text-violet-600">
              Instances
            </Link>
            <a href="/api-docs/" target="_blank" className="hover:text-violet-600">
              API
            </a>
            <span className="text-zinc-500">{user?.username}</span>
            <button onClick={onLogout} className="rounded border px-3 py-1 text-sm hover:bg-zinc-100">
              Sign out
            </button>
          </nav>
        </div>
      </header>
      <main className="mx-auto max-w-6xl p-4">
        <Outlet />
      </main>
    </div>
  );
}
