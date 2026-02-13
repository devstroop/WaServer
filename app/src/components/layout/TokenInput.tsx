import { useState, useEffect } from "react"
import { useMutation, useQuery } from "@tanstack/react-query"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Button,
  Input,
  Label,
  toast,
} from "@/components/ui"
import { useSettingsStore } from "@/store"
import { apiClient } from "@/lib/api"
import { Key, MessageSquare, ArrowRight, User, Lock, Loader2 } from "lucide-react"

type AuthMode = "loading" | "login" | "token"

export function TokenInput() {
  const [tokenInput, setTokenInput] = useState("")
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")
  const [authMode, setAuthMode] = useState<AuthMode>("loading")
  
  const { setApiToken, setAuthTokens, setLocalAuthEnabled } = useSettingsStore()

  // Check if local auth is enabled
  const { data: localAuthStatus, isLoading } = useQuery({
    queryKey: ["localAuthStatus"],
    queryFn: () => apiClient.getLocalAuthStatus(),
    retry: 1,
  })

  useEffect(() => {
    if (!isLoading) {
      if (localAuthStatus?.local_auth_enabled) {
        setAuthMode("login")
        setLocalAuthEnabled(true)
      } else {
        setAuthMode("token")
        setLocalAuthEnabled(false)
      }
    }
  }, [localAuthStatus, isLoading, setLocalAuthEnabled])

  // Login mutation
  const loginMutation = useMutation({
    mutationFn: async () => {
      return apiClient.login({ username, password })
    },
    onSuccess: (data) => {
      setAuthTokens({
        accessToken: data.access_token,
        refreshToken: data.refresh_token,
        expiresAt: Date.now() / 1000 + data.expires_in,
        username: data.username,
      })
      toast({
        title: "Welcome!",
        description: `Logged in as ${data.username}`,
        variant: "success",
      })
    },
    onError: (error) => {
      toast({
        title: "Login Failed",
        description: error instanceof Error ? error.message : "Invalid credentials",
        variant: "destructive",
      })
    },
  })

  const handleTokenSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (tokenInput.trim()) {
      setApiToken(tokenInput.trim())
      toast({
        title: "Connected!",
        description: "API token saved successfully",
        variant: "success",
      })
    }
  }

  const handleLoginSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (username && password) {
      loginMutation.mutate()
    }
  }

  if (authMode === "loading" || isLoading) {
    return (
      <Card className="w-full max-w-md shadow-2xl border-0 bg-card/95 backdrop-blur">
        <CardContent className="flex items-center justify-center py-16">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className="w-full max-w-md shadow-2xl border-0 bg-card/95 backdrop-blur">
      <CardHeader className="text-center pb-2">
        <div className="mx-auto mb-4 flex items-center justify-center w-16 h-16 rounded-2xl bg-whatsapp shadow-lg">
          <MessageSquare className="h-8 w-8 text-white" />
        </div>
        <CardTitle className="text-2xl font-bold">Welcome to WAS</CardTitle>
        <CardDescription className="text-base">
          WhatsApp Automation Server
        </CardDescription>
      </CardHeader>
      <CardContent className="pt-4">
        {authMode === "login" ? (
          // Login Form
          <form onSubmit={handleLoginSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="username" className="text-sm font-medium">
                Username
              </Label>
              <div className="relative">
                <User className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  id="username"
                  type="text"
                  placeholder="Enter username"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  className="pl-10 h-12 bg-background/50"
                  autoComplete="username"
                  autoFocus
                  disabled={loginMutation.isPending}
                />
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="password" className="text-sm font-medium">
                Password
              </Label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  id="password"
                  type="password"
                  placeholder="Enter password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="pl-10 h-12 bg-background/50"
                  autoComplete="current-password"
                  disabled={loginMutation.isPending}
                />
              </div>
            </div>
            <Button
              type="submit"
              variant="whatsapp"
              className="w-full h-12 text-base font-medium gap-2"
              disabled={!username || !password || loginMutation.isPending}
            >
              {loginMutation.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Logging in...
                </>
              ) : (
                <>
                  Login
                  <ArrowRight className="h-4 w-4" />
                </>
              )}
            </Button>
            
            <div className="text-center">
              <button
                type="button"
                onClick={() => setAuthMode("token")}
                className="text-xs text-muted-foreground hover:text-foreground transition-colors"
              >
                Or use static API token instead
              </button>
            </div>
          </form>
        ) : (
          // Token Form
          <form onSubmit={handleTokenSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="apiToken" className="text-sm font-medium">
                API Token
              </Label>
              <div className="relative">
                <Key className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  id="apiToken"
                  type="password"
                  placeholder="Enter your API token"
                  value={tokenInput}
                  onChange={(e) => setTokenInput(e.target.value)}
                  className="pl-10 h-12 bg-background/50"
                  autoFocus
                />
              </div>
              <p className="text-xs text-muted-foreground">
                Find your token in{" "}
                <code className="px-1.5 py-0.5 bg-muted rounded text-[10px] font-mono">
                  config/app.toml
                </code>{" "}
                → <code className="px-1.5 py-0.5 bg-muted rounded text-[10px] font-mono">[auth]</code>{" "}
                section
              </p>
            </div>
            <Button
              type="submit"
              variant="whatsapp"
              className="w-full h-12 text-base font-medium gap-2"
              disabled={!tokenInput.trim()}
            >
              Connect to Server
              <ArrowRight className="h-4 w-4" />
            </Button>
            
            {localAuthStatus?.local_auth_enabled && (
              <div className="text-center">
                <button
                  type="button"
                  onClick={() => setAuthMode("login")}
                  className="text-xs text-muted-foreground hover:text-foreground transition-colors"
                >
                  Or login with username and password
                </button>
              </div>
            )}
          </form>
        )}
        
        <div className="mt-6 pt-6 border-t">
          <p className="text-xs text-center text-muted-foreground">
            WAS v0.2.0 • Built with ❤️ for automation
          </p>
        </div>
      </CardContent>
    </Card>
  )
}
