import { useEffect, useRef } from 'react';
import { useUIStore } from '@/stores';

const BREAKPOINT_LG = 1024;
const BREAKPOINT_XL = 1280;

export function useSidebar() {
  const sidebarCollapsed = useUIStore((state) => state.sidebarCollapsed);
  const sidebarOpen = useUIStore((state) => state.sidebarOpen);
  const toggleSidebar = useUIStore((state) => state.toggleSidebar);
  const setSidebarCollapsed = useUIStore((state) => state.setSidebarCollapsed);
  const setSidebarOpen = useUIStore((state) => state.setSidebarOpen);
  
  const initializedRef = useRef(false);

  // Auto-collapse sidebar on smaller screens - only on initial mount
  useEffect(() => {
    if (initializedRef.current) return;
    initializedRef.current = true;
    
    const width = window.innerWidth;
    if (width >= BREAKPOINT_LG && width < BREAKPOINT_XL) {
      setSidebarCollapsed(true);
    }
  }, [setSidebarCollapsed]);

  // Close mobile nav when resizing to desktop
  useEffect(() => {
    const handleResize = () => {
      const width = window.innerWidth;
      if (width >= BREAKPOINT_LG && sidebarOpen) {
        setSidebarOpen(false);
      }
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [setSidebarOpen, sidebarOpen]);

  return {
    collapsed: sidebarCollapsed,
    mobileOpen: sidebarOpen,
    toggle: toggleSidebar,
    setCollapsed: setSidebarCollapsed,
    setMobileOpen: setSidebarOpen,
  };
}
