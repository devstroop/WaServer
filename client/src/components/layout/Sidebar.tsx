import { useState, useCallback } from 'react';
import { NavLink } from 'react-router-dom';
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
  PanelLeftClose,
  PanelLeft,
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/Button';
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

export function Sidebar() {
  const { collapsed, toggle } = useSidebar();
  const [hoverExpanded, setHoverExpanded] = useState(false);

  // Effective state: expanded via hover or not collapsed
  const isExpanded = !collapsed || hoverExpanded;

  const handleMouseEnter = useCallback(() => {
    if (collapsed) {
      setHoverExpanded(true);
    }
  }, [collapsed]);

  const handleMouseLeave = useCallback(() => {
    setHoverExpanded(false);
  }, []);

  return (
    <aside
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      className={cn(
        'fixed left-0 top-0 z-40 h-screen bg-card border-r transition-all duration-300 hidden lg:flex flex-col',
        isExpanded ? 'w-64' : 'w-16',
        hoverExpanded && 'shadow-xl'
      )}
    >
      <div className="flex h-16 items-center justify-between border-b px-4">
        {isExpanded && (
          <span className="text-xl font-bold text-primary">WAS</span>
        )}
        <Button
          variant="ghost"
          size="icon"
          onClick={toggle}
          className={cn('hover:bg-accent', !isExpanded && 'mx-auto')}
          title={collapsed ? 'Pin sidebar open' : 'Collapse sidebar'}
        >
          {collapsed ? (
            <PanelLeft className="h-5 w-5" />
          ) : (
            <PanelLeftClose className="h-5 w-5" />
          )}
        </Button>
      </div>

      <nav className="flex-1 space-y-1 p-2 overflow-y-auto scrollbar-thin">
        {navItems.map((item) => (
          <NavLink
            key={item.href}
            to={item.href}
            className={({ isActive }) =>
              cn(
                'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground',
                isActive
                  ? 'bg-accent text-accent-foreground'
                  : 'text-muted-foreground',
                !isExpanded && 'justify-center px-2'
              )
            }
            title={!isExpanded ? item.label : undefined}
          >
            <item.icon className="h-5 w-5 shrink-0" />
            {isExpanded && <span>{item.label}</span>}
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
