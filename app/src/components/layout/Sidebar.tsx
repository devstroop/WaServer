import { NavLink } from "react-router-dom"
import { cn } from "@/lib/utils"
import { useSettingsStore } from "@/store"
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui"
import {
  LayoutDashboard,
  MessageSquare,
  Settings,
  Webhook,
  BookOpen,
  ExternalLink,
  Key,
} from "lucide-react"

// Navigation items
const navItems = [
  { to: "/", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/chat", icon: MessageSquare, label: "Chats" },
  { to: "/webhooks", icon: Webhook, label: "Webhooks" },
  { to: "/access-tokens", icon: Key, label: "Access Tokens" },
  { to: "/settings", icon: Settings, label: "Settings" },
]

// External links
const externalLinks = [
  { href: "/swagger-ui", icon: BookOpen, label: "API Docs" },
]

export function Sidebar() {
  const { sidebarCollapsed } = useSettingsStore()

  return (
    <aside
      className={cn(
        "flex flex-col h-full bg-card border-r transition-all duration-300 ease-in-out pt-2",
        sidebarCollapsed ? "w-16" : "w-60"
      )}
    >
      {/* Navigation */}
      <nav className="flex-1 p-2 space-y-1 overflow-y-auto">
        {!sidebarCollapsed && (
          <div className="px-3 py-2 animate-in fade-in-0 duration-200">
            <span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
              Navigation
            </span>
          </div>
        )}
        
        {navItems.map((item) => (
          <Tooltip key={item.to}>
            <TooltipTrigger asChild>
              <NavLink
                to={item.to}
                className={({ isActive }) =>
                  cn(
                    "flex items-center rounded-lg font-medium transition-all duration-200",
                    "hover:bg-accent hover:text-accent-foreground",
                    "active:scale-[0.98]",
                    sidebarCollapsed 
                      ? "w-12 h-12 justify-center mx-auto" 
                      : "gap-3 px-3 py-2.5",
                    isActive
                      ? "bg-primary text-primary-foreground shadow-sm"
                      : "text-muted-foreground"
                  )
                }
              >
                <item.icon className={cn(
                  "shrink-0 transition-all",
                  sidebarCollapsed ? "h-5 w-5" : "h-5 w-5"
                )} />
                {!sidebarCollapsed && (
                  <span className="text-sm truncate animate-in fade-in-0 slide-in-from-left-2 duration-200">
                    {item.label}
                  </span>
                )}
              </NavLink>
            </TooltipTrigger>
            {sidebarCollapsed && (
              <TooltipContent side="right">
                {item.label}
              </TooltipContent>
            )}
          </Tooltip>
        ))}

        {/* Divider */}
        <div className={cn(
          "my-3",
          sidebarCollapsed ? "mx-2" : "mx-3"
        )}>
          <div className="h-px bg-border" />
        </div>

        {/* External Links */}
        {!sidebarCollapsed && (
          <div className="px-3 py-2 animate-in fade-in-0 duration-200">
            <span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
              Resources
            </span>
          </div>
        )}
        
        {externalLinks.map((item) => (
          <Tooltip key={item.href}>
            <TooltipTrigger asChild>
              <a
                href={item.href}
                target="_blank"
                rel="noopener noreferrer"
                className={cn(
                  "flex items-center rounded-lg font-medium transition-all duration-200",
                  "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
                  "active:scale-[0.98]",
                  sidebarCollapsed 
                    ? "w-12 h-12 justify-center mx-auto" 
                    : "gap-3 px-3 py-2.5"
                )}
              >
                <item.icon className="h-5 w-5 shrink-0" />
                {!sidebarCollapsed && (
                  <span className="text-sm truncate flex-1 animate-in fade-in-0 slide-in-from-left-2 duration-200">
                    {item.label}
                  </span>
                )}
                {!sidebarCollapsed && (
                  <ExternalLink className="h-3 w-3 opacity-50 animate-in fade-in-0 duration-200" />
                )}
              </a>
            </TooltipTrigger>
            {sidebarCollapsed && (
              <TooltipContent side="right">
                {item.label}
              </TooltipContent>
            )}
          </Tooltip>
        ))}
      </nav>

    </aside>
  )
}
