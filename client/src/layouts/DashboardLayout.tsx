import type { ReactNode } from 'react';
import { cn } from '@/lib/utils';
import { Sidebar } from '@/components/layout/Sidebar';
import { MobileNav } from '@/components/layout/MobileNav';
import { Topbar } from '@/components/layout/Topbar';
import { useSidebar } from '@/hooks/useSidebar';

interface DashboardLayoutProps {
  children: ReactNode;
}

export function DashboardLayout({ children }: DashboardLayoutProps) {
  const { collapsed } = useSidebar();

  return (
    <div className="min-h-screen bg-background">
      <Sidebar />
      <MobileNav />
      <div
        className={cn(
          'transition-all duration-300',
          collapsed ? 'lg:ml-16' : 'lg:ml-64'
        )}
      >
        <Topbar />
        <main>{children}</main>
      </div>
    </div>
  );
}
