import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { apiClient } from "@/lib/api"
import { useSettingsStore } from "@/store"
import {
  Button,
  toast,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui"
import {
  Link,
  Unlink,
  Loader2,
  Smartphone,
  Moon,
  Sun,
  LogOut,
  User,
  MessageSquare,
  PanelLeftClose,
  PanelLeft,
} from "lucide-react"
import { AuthPanel } from "@/components/auth"

export function Header() {
  const queryClient = useQueryClient()
  const { 
    apiToken, 
    clearApiToken, 
    authTokens,
    clearAuthTokens,
    localAuthEnabled,
    theme, 
    setTheme,
    authDialogOpen,
    setAuthDialogOpen,
    sidebarCollapsed,
    toggleSidebar,
  } = useSettingsStore()

  // Check if we have either auth method
  const hasToken = !!apiToken || !!authTokens

  // Get auth status
  const { data: authStatus, isLoading } = useQuery({
    queryKey: ["authStatus"],
    queryFn: () => apiClient.getAuthStatus(),
    refetchInterval: 5000,
    enabled: hasToken,
  })

  const isChecking = authStatus?.status === "checking" || isLoading
  const isDeviceLinked = authStatus?.authenticated === true
  const needsAuth = authStatus?.status === "not_authenticated"

  const logoutMutation = useMutation({
    mutationFn: () => apiClient.logout(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["authStatus"] })
      queryClient.invalidateQueries({ queryKey: ["health"] })
      toast({
        title: "Device unlinked",
        description: "WhatsApp device has been disconnected",
      })
    },
    onError: (error: Error) => {
      toast({
        title: "Unlink failed",
        description: error.message,
        variant: "destructive",
      })
    },
  })

  const toggleTheme = () => {
    setTheme(theme === "light" ? "dark" : "light")
  }

  const handleLogout = async () => {
    // If using JWT auth, revoke refresh token
    if (authTokens?.refreshToken && localAuthEnabled) {
      try {
        await apiClient.localLogout(authTokens.refreshToken)
      } catch {
        // Ignore errors, we're logging out anyway
      }
    }
    
    // Clear tokens
    clearApiToken()
    clearAuthTokens()
    queryClient.clear()
    
    toast({
      title: "Logged out",
      description: "You have been logged out successfully",
    })
  }

  return (
    <>
      <header className="h-14 border-b bg-card/50 backdrop-blur-sm flex items-center justify-between px-4 sticky top-0 z-40">
        {/* Left side - Logo and sidebar toggle */}
        <div className="flex items-center gap-3">
          {/* Sidebar Toggle */}
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={toggleSidebar}
                className="h-9 w-9 rounded-lg"
              >
                {sidebarCollapsed ? (
                  <PanelLeft className="h-4 w-4" />
                ) : (
                  <PanelLeftClose className="h-4 w-4" />
                )}
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
              {sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}
            </TooltipContent>
          </Tooltip>

          {/* Logo */}
          <div className="flex items-center gap-2.5">
            <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-whatsapp">
              <MessageSquare className="h-4 w-4 text-white" />
            </div>
            <div className="hidden sm:block">
              <h1 className="font-bold text-sm leading-tight">WAS</h1>
              <p className="text-[10px] text-muted-foreground">WhatsApp Server</p>
            </div>
          </div>
        </div>

        {/* Right side - Actions */}
        <div className="flex items-center gap-2">
          {/* Theme Toggle */}
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={toggleTheme}
                className="h-9 w-9 rounded-lg"
              >
                {theme === "light" ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
              Theme: {theme}
            </TooltipContent>
          </Tooltip>

          {/* Device Status */}
          {hasToken && (
            <>
              {isChecking && !isDeviceLinked && !needsAuth && (
                <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-muted/50 text-muted-foreground text-sm">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  <span className="hidden sm:inline">Connecting...</span>
                </div>
              )}

              {needsAuth && (
                <Button
                  variant="default"
                  size="sm"
                  onClick={() => setAuthDialogOpen(true)}
                  className="gap-2 bg-whatsapp hover:bg-whatsapp/90"
                >
                  <Link className="h-4 w-4" />
                  <span className="hidden sm:inline">Link Device</span>
                </Button>
              )}

              {isDeviceLinked && (
                <div className="flex items-center gap-2">
                  <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-whatsapp/10 text-whatsapp text-sm">
                    <Smartphone className="h-4 w-4" />
                    <span className="hidden sm:inline font-medium">
                      {authStatus?.phone_number || "Connected"}
                    </span>
                    <span className="sm:hidden">●</span>
                  </div>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => logoutMutation.mutate()}
                        disabled={logoutMutation.isPending}
                        className="h-9 w-9 rounded-lg text-muted-foreground hover:text-destructive hover:bg-destructive/10"
                      >
                        {logoutMutation.isPending ? (
                          <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                          <Unlink className="h-4 w-4" />
                        )}
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent side="bottom">
                      Unlink WhatsApp Device
                    </TooltipContent>
                  </Tooltip>
                </div>
              )}

              {/* User info if logged in with JWT */}
              {authTokens && (
                <div className="hidden sm:flex items-center gap-2 px-3 py-1.5 rounded-lg bg-muted/50 text-muted-foreground text-sm">
                  <User className="h-4 w-4" />
                  <span className="font-medium">{authTokens.username}</span>
                </div>
              )}

              {/* Logout from UI */}
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={handleLogout}
                    className="h-9 w-9 rounded-lg text-muted-foreground hover:text-destructive"
                  >
                    <LogOut className="h-4 w-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom">
                  {authTokens ? "Logout" : "Clear API Token"}
                </TooltipContent>
              </Tooltip>
            </>
          )}
        </div>
      </header>

      {/* Auth Dialog */}
      <Dialog open={authDialogOpen} onOpenChange={setAuthDialogOpen}>
        <DialogContent className="max-w-md p-0 overflow-hidden" showClose={true}>
          <DialogHeader className="p-6 pb-2">
            <DialogTitle className="flex items-center gap-2">
              <Smartphone className="h-5 w-5 text-whatsapp" />
              Link WhatsApp Device
            </DialogTitle>
            <DialogDescription>
              Scan QR code or use phone number to connect
            </DialogDescription>
          </DialogHeader>
          <div className="px-6 pb-6">
            <AuthPanel 
              onAuthenticated={() => setAuthDialogOpen(false)} 
              compact 
            />
          </div>
        </DialogContent>
      </Dialog>
    </>
  )
}
