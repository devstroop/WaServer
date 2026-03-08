import { NavLink, useNavigate } from 'react-router-dom';
import {
  LayoutDashboard,
  Server,
  Smartphone,
  MessageSquare,
  Megaphone,
  Users,
  BarChart3,
  Key,
  FileText,
  Settings,
  LogOut,
  X,
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/Button';
import { ThemeToggle } from '@/theme';
import { useAuthStore } from '@/stores';
import { useSidebar } from '@/hooks/useSidebar';

interface NavItem {
  label: string;
  href: string;
  icon: React.FC<React.SVGProps<SVGSVGElement>>;
}

const navItems: NavItem[] = [
  { label: 'Dashboard', href: '/dashboard', icon: LayoutDashboard },
  { label: 'Servers', href: '/servers', icon: Server },
  { label: 'Sessions', href: '/sessions', icon: Smartphone },
  { label: 'Messages', href: '/messages', icon: MessageSquare },
  { label: 'Campaigns', href: '/campaigns', icon: Megaphone },
  { label: 'Contacts', href: '/contacts', icon: Users },
  { label: 'Analytics', href: '/analytics', icon: BarChart3 },
  { label: 'API Keys', href: '/api-keys', icon: Key },
  { label: 'Logs', href: '/logs', icon: FileText },
  { label: 'Settings', href: '/settings', icon: Settings },
];

export function MobileNav() {
  const navigate = useNavigate();
  const logout = useAuthStore((state) => state.logout);
  const { mobileOpen, setMobileOpen } = useSidebar();

  const handleLogout = () => {
    logout();
    navigate('/login');
    setMobileOpen(false);
  };

  const handleNavClick = () => {
    setMobileOpen(false);
  };

  if (!mobileOpen) return null;

  return (
    <>
      <div
        className="fixed inset-0 z-40 bg-background/80 backdrop-blur-sm lg:hidden"
        onClick={() => setMobileOpen(false)}
      />
      <aside className="fixed left-0 top-0 z-50 h-screen w-64 bg-card border-r lg:hidden flex flex-col">
        <div className="flex h-16 items-center justify-between border-b px-4">
          <span className="text-xl font-bold text-primary">WAS</span>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setMobileOpen(false)}
          >
            <X className="h-4 w-4" />
          </Button>
        </div>

        <nav className="flex-1 space-y-1 p-2 overflow-y-auto scrollbar-thin">
          {navItems.map((item) => (
            <NavLink
              key={item.href}
              to={item.href}
              onClick={handleNavClick}
              className={({ isActive }) =>
                cn(
                  'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground',
                  isActive
                    ? 'bg-accent text-accent-foreground'
                    : 'text-muted-foreground'
                )
              }
            >
              <item.icon className="h-5 w-5" />
              <span>{item.label}</span>
            </NavLink>
          ))}
        </nav>

        <div className="border-t p-2 space-y-1">
          <div className="px-3 mb-2">
            <ThemeToggle />
          </div>
          <Button
            variant="ghost"
            className="w-full justify-start gap-3"
            onClick={handleLogout}
          >
            <LogOut className="h-5 w-5" />
            <span>Logout</span>
          </Button>
        </div>
      </aside>
    </>
  );
}
