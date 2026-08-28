import { type JSX, useEffect, useState } from "react";
import { Link, Outlet, useNavigate } from "react-router-dom";
import { Avatar, Button, Icon, Sidebar } from "@devstroop/react-uikit";
import { useAuth } from "../hooks/useAuth";

type NavLink = { href: string; label: string; icon: "home" | "info" | "folder" | "users" | "key" | "settings" };
type NavGroup = { label: string; links: NavLink[] };

const NAV: NavGroup[] = [
  { label: "Overview", links: [{ href: "/admin/dashboard", label: "Dashboard", icon: "home" }, { href: "/admin/metrics", label: "Metrics", icon: "info" }] },
  { label: "Instances", links: [{ href: "/admin/instances", label: "Instances", icon: "folder" }] },
  { label: "Access", links: [{ href: "/admin/users", label: "Users", icon: "users" }, { href: "/settings/api-keys", label: "API Keys", icon: "key" }] },
  { label: "System", links: [{ href: "/admin/settings", label: "Settings", icon: "settings" }] },
];

export default function AdminLayout(): JSX.Element {
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
  const isActive = (href: string) => path === href || path.startsWith(href + "/");
  const onLogout = async () => {
    await logout();
    nav("/auth/login");
  };
  return (
    <div className="flex min-h-screen bg-zinc-50">
      <Sidebar position="left" className="w-64 shrink-0 border-r bg-white !p-0">
        <Link to="/admin/dashboard" className="flex h-14 items-center gap-3 border-b px-4 font-bold tracking-tight">
          <span className="flex h-8 w-8 items-center justify-center rounded-lg bg-zinc-900 text-xs font-bold text-white">W</span>
          <span>WAS Admin</span>
          <span className="ml-auto text-[10px] font-normal text-zinc-400">v0.6.0</span>
        </Link>
        <nav className="flex-1 space-y-6 overflow-y-auto p-3">
          {NAV.map((g) => (
            <div key={g.label}>
              <div className="px-2 py-1 text-[11px] font-semibold uppercase tracking-wider text-zinc-400">{g.label}</div>
              <div className="mt-1 space-y-1">
                {g.links.map((l) => (
                  <Link
                    key={l.href}
                    to={l.href}
                    className={`flex items-center gap-2.5 rounded-md px-2.5 py-2 text-sm transition ${isActive(l.href) ? "bg-zinc-900 font-medium text-white shadow-sm" : "text-zinc-600 hover:bg-zinc-100 hover:text-zinc-900"}`}
                  >
                    <Icon name={l.icon} size={16} className={isActive(l.href) ? "text-white" : "text-zinc-400"} />
                    {l.label}
                  </Link>
                ))}
              </div>
            </div>
          ))}
        </nav>
        <div className="border-t bg-white p-3">
          {user && (
            <div className="mb-3 flex items-center gap-3 rounded-lg border bg-zinc-50 p-2.5">
              <Avatar name={user.username} size="sm" />
              <div className="min-w-0 flex-1">
                <div className="truncate text-sm font-medium leading-tight">{user.username}</div>
                <div className="text-xs capitalize text-zinc-500">{user.role} • {user.email ?? "no email"}</div>
              </div>
              <span className="h-2 w-2 rounded-full bg-emerald-500" title="active" />
            </div>
          )}
          <Button variant="ghost" size="sm" fullWidth onClick={onLogout} className="justify-start gap-2">
            <Icon name="close" size={14} />
            Sign out
          </Button>
        </div>
      </Sidebar>
      <main className="flex-1 bg-[#F8FAFC] p-6">
        <Outlet />
      </main>
    </div>
  );
}
