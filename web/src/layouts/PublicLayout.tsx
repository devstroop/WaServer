import { type JSX, useEffect, useState } from "react";
import { Link, Outlet, useNavigate } from "react-router-dom";
import { Button, Layout, Header, Footer, ThemeSwitcher } from "@devstroop/react-uikit";
import { useAuth } from "../hooks/useAuth";

export default function PublicLayout(): JSX.Element {
  const { user } = useAuth();
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
  return (
    <Layout>
      <Header className="w-full border-b bg-white/80 backdrop-blur">
        <div className="mx-auto flex h-[1.5rem] w-full max-w-7xl items-center justify-between gap-4 px-4 sm:px-6 lg:px-8">
          <Link to="/" className="flex items-center gap-2.5 font-bold tracking-tight">
            <span className="flex h-7 w-7 items-center justify-center rounded bg-primary text-xs font-bold text-primary-fg">W</span>
            <span>WAS</span>
            <span className="hidden text-sm font-normal text-zinc-500 sm:inline">WhatsApp Server</span>
          </Link>
          <nav className="hidden items-center gap-1 md:flex">
            <Link to="/#features" className={`rounded-full px-3 py-1.5 text-sm font-medium transition hover:bg-zinc-100 ${isActive("/#features") ? "bg-zinc-100" : ""}`}>Features</Link>
            <a href="/api-docs/" target="_blank" className="rounded-full px-3 py-1.5 text-sm font-medium transition hover:bg-zinc-100">API Docs</a>
          </nav>
          <div className="flex items-center gap-2">
            {user ? (
              <Button variant="secondary" size="sm" onClick={() => nav(user.role === "admin" ? "/admin/dashboard" : "/dashboard")}>
                Dashboard
              </Button>
            ) : (
              <>
                <Button variant="ghost" size="sm" onClick={() => nav("/auth/login")}>Sign in</Button>
                <Button variant="primary" size="sm" onClick={() => nav("/auth/register")}>Create account</Button>
              </>
            )}
            <div className="ml-1 border-l pl-2">
              <ThemeSwitcher />
            </div>
          </div>
        </div>
      </Header>
      <main className="w-full flex-1 bg-[#F8FAFC]">
        <Outlet />
      </main>
      <Footer className="w-full border-t bg-white">
        <div className="mx-auto w-full max-w-7xl">
          <div className="flex flex-col items-center justify-between gap-4 text-sm md:flex-row">
            <div className="flex items-center gap-2 font-semibold">
              <span className="flex h-6 w-6 items-center justify-center rounded bg-zinc-900 text-xs text-white">W</span>
              WAS — WhatsApp Server
            </div>
            <div className="flex items-center gap-4 text-xs text-zinc-500">
              <a href="/api-docs/" className="hover:text-zinc-900">API Docs</a>
              <a href="#" className="hover:text-zinc-900">Privacy</a>
              <a href="#" className="hover:text-zinc-900">Terms</a>
              <span>© {new Date().getFullYear()} Devstroop</span>
            </div>
          </div>
        </div>
      </Footer>
    </Layout>
  );
}
