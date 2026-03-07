import { NavLink, useNavigate } from 'react-router-dom';
import { LayoutDashboard, MessageSquare, Server, Settings, LogOut, Menu, X, ChevronLeft, ChevronRight } from 'lucide-react';
import { cn } from '@/lib/utils';
import { ROUTES } from '@/lib/constants';
import { Button } from '@/components/ui/Button';
import { ThemeToggle } from '@/theme';
import { useUIStore, useAuthStore } from '@/stores';

interface NavItem {
  label: string;
  href: string;
  icon: React.ComponentType<{ className?: string | undefined }>;
}

const navItems: NavItem[] = [
  { label: 'Dashboard', href: ROUTES.DASHBOARD, icon: LayoutDashboard },
  { label: 'Instances', href: ROUTES.INSTANCES, icon: Server },
  { label: 'Messages', href: ROUTES.MESSAGES, icon: MessageSquare },
  { label: 'Settings', href: ROUTES.SETTINGS, icon: Settings },
];

interface SidebarProps {
  collapsed: boolean;
  onToggle: () => void;
}

function Sidebar({ collapsed, onToggle }: SidebarProps) {
  const navigate = useNavigate();
  const logout = useAuthStore((state) => state.logout);

  const handleLogout = () => {
    logout();
    navigate(ROUTES.LOGIN);
  };

  return (
    <aside className={cn('fixed left-0 top-0 z-40 h-screen bg-card border-r transition-all duration-300', collapsed ? 'w-16' : 'w-64')}>
      <div className="flex h-full flex-col">
        <div className="flex h-16 items-center justify-between border-b px-4">
          {!collapsed && <span className="text-xl font-bold text-primary">WAS</span>}
          <Button variant="ghost" size="icon" onClick={onToggle} className="ml-auto">
            {collapsed ? <ChevronRight className="h-4 w-4" /> : <ChevronLeft className="h-4 w-4" />}
          </Button>
        </div>
        <nav className="flex-1 space-y-1 p-2">
          {navItems.map((item) => (
            <NavLink key={item.href} to={item.href} className={({ isActive }) =>
              cn('flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground',
                isActive ? 'bg-accent text-accent-foreground' : 'text-muted-foreground', collapsed && 'justify-center')}>
              <item.icon className="h-5 w-5 shrink-0" />
              {!collapsed && <span>{item.label}</span>}
            </NavLink>
          ))}
        </nav>
        <div className="border-t p-2 space-y-1">
          <div className={cn('flex items-center', collapsed ? 'justify-center' : 'px-3')}><ThemeToggle /></div>
          <Button variant="ghost" className={cn('w-full justify-start gap-3', collapsed && 'justify-center')} onClick={handleLogout}>
            <LogOut className="h-5 w-5" />
            {!collapsed && <span>Logout</span>}
          </Button>
        </div>
      </div>
    </aside>
  );
}

function MobileNav({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  const navigate = useNavigate();
  const logout = useAuthStore((state) => state.logout);
  const handleLogout = () => { logout(); navigate(ROUTES.LOGIN); onClose(); };

  if (!isOpen) return null;

  return (
    <>
      <div className="fixed inset-0 z-40 bg-background/80 backdrop-blur-sm lg:hidden" onClick={onClose} />
      <aside className="fixed left-0 top-0 z-50 h-screen w-64 bg-card border-r lg:hidden">
        <div className="flex h-16 items-center justify-between border-b px-4">
          <span className="text-xl font-bold text-primary">WAS</span>
          <Button variant="ghost" size="icon" onClick={onClose}><X className="h-4 w-4" /></Button>
        </div>
        <nav className="flex-1 space-y-1 p-2">
          {navItems.map((item) => (
            <NavLink key={item.href} to={item.href} onClick={onClose}
              className={({ isActive }) => cn('flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors hover:bg-accent',
                isActive ? 'bg-accent text-accent-foreground' : 'text-muted-foreground')}>
              <item.icon className="h-5 w-5" /><span>{item.label}</span>
            </NavLink>
          ))}
        </nav>
        <div className="border-t p-2">
          <div className="px-3 mb-2"><ThemeToggle /></div>
          <Button variant="ghost" className="w-full justify-start gap-3" onClick={handleLogout}><LogOut className="h-5 w-5" /><span>Logout</span></Button>
        </div>
      </aside>
    </>
  );
}

interface MainLayoutProps {
  children: React.ReactNode;
}

export function MainLayout({ children }: MainLayoutProps) {
  const { sidebarCollapsed, sidebarOpen, toggleSidebar, setSidebarOpen } = useUIStore();

  return (
    <div className="min-h-screen bg-background">
      <div className="hidden lg:block"><Sidebar collapsed={sidebarCollapsed} onToggle={toggleSidebar} /></div>
      <MobileNav isOpen={sidebarOpen} onClose={() => setSidebarOpen(false)} />
      <header className="sticky top-0 z-30 flex h-16 items-center gap-4 border-b bg-background px-4 lg:hidden">
        <Button variant="ghost" size="icon" onClick={() => setSidebarOpen(true)}><Menu className="h-5 w-5" /></Button>
        <span className="text-xl font-bold text-primary">WAS</span>
      </header>
      <main className={cn('transition-all duration-300 p-6', sidebarCollapsed ? 'lg:ml-16' : 'lg:ml-64')}>{children}</main>
    </div>
  );
}
