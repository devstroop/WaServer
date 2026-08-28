import { type JSX, useEffect, useState } from "react";
import { Link, Outlet, useNavigate } from "react-router-dom";
import { Avatar, Button, Sidebar, ThemeSwitcher } from "@devstroop/react-uikit";
import { useAuth } from "../hooks/useAuth";

const NAV = [
  { href: "/dashboard", label: "Dashboard" },
  { href: "/dashboard/instances", label: "My Instances" },
  { href: "/settings/api-keys", label: "API Keys" },
  { href: "/settings", label: "Settings" },
];

export default function UserLayout(): JSX.Element {
  const { user, logout } = useAuth();
  const nav = useNavigate();
  const [path, setPath] = useState(window.location.pathname);
  useEffect(() => {
    const sync = () => setPath(window.location.pathname);
    window.addEventListener("popstate", sync);
    window.addEventListener("spa:navigate", sync as EventListener);
    return () => {
      window.removeEventListener("popstate", sync);
      window.removeEventListener("spa:navigate", sync as EventListener);
    };
  }, []);
  const isActive = (h: string) => path === h || path.startsWith(h + "/");
  const onLogout = async () => {
    await logout();
    nav("/auth/login");
  };
  return (
    <div className="flex min-h-screen">
      <Sidebar position="left" className="w-64 shrink-0 border-r bg-white !p-0">
        <Link to="/dashboard" className="flex h-14 items-center gap-2 border-b px-4 font-bold tracking-tight">
          WAS
        </Link>
        <nav className="space-y-1 p-3">
          {NAV.map((l) => (
            <Link key={l.href} to={l.href} className={`block rounded px-2 py-1.5 text-sm hover:bg-zinc-100 ${isActive(l.href) ? "bg-zinc-100 font-medium" : ""}`}>
              {l.label}
            </Link>
          ))}
        </nav>
        <div className="mt-auto border-t p-3">
          {user && (
            <div className="mb-2 flex items-center gap-2">
              <Avatar name={user.username} size="sm" />
              <span className="text-sm">{user.username}</span>
            </div>
          )}
          <Button variant="ghost" size="sm" fullWidth onClick={onLogout}>
            Sign out
          </Button>
          <div className="mt-2 flex justify-center">
            <ThemeSwitcher />
          </div>
        </div>
      </Sidebar>
      <main className="flex-1 bg-[#F8FAFC] p-6">
        <Outlet />
      </main>
    </div>
  );
}
