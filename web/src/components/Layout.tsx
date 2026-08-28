import { Link, Outlet, useNavigate } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';

export default function Layout() {
  const { user, logout } = useAuth();
  const nav = useNavigate();
  const onLogout = async () => {
    await logout();
    nav('/auth/login');
  };
  return (
    <div className="min-h-screen bg-[#F0F2F5] text-[#111B21]">
      <header className="sticky top-0 z-10 border-b border-[#128C7E] bg-[#128C7E] text-white">
        <div className="mx-auto flex max-w-6xl items-center justify-between px-4 py-3">
          <Link to="/" className="font-semibold tracking-tight text-white">
            WAS <span className="text-[#DCF8C6]">Admin</span>
          </Link>
          <nav className="flex items-center gap-4 text-sm text-white">
            <Link to="/dashboard" className="hover:text-[#DCF8C6]">
              Dashboard
            </Link>
            <Link to="/dashboard/instances" className="hover:text-[#DCF8C6]">
              Instances
            </Link>
            <a href="/api-docs/" target="_blank" className="hover:text-[#DCF8C6]">
              API
            </a>
            <span className="text-white/80">{user?.username}</span>
            <button onClick={onLogout} className="rounded border border-white/20 bg-white/10 px-3 py-1 text-sm text-white hover:bg-white/20">
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
