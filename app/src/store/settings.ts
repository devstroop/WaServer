import { create } from "zustand"
import { persist } from "zustand/middleware"

interface AuthTokens {
  accessToken: string
  refreshToken: string
  expiresAt: number // Unix timestamp
  username: string
}

interface SettingsState {
  // Static API Token (legacy support)
  apiToken: string
  setApiToken: (token: string) => void
  clearApiToken: () => void
  
  // Local Auth (JWT tokens)
  authTokens: AuthTokens | null
  setAuthTokens: (tokens: AuthTokens | null) => void
  clearAuthTokens: () => void
  isLoggedIn: () => boolean
  getActiveToken: () => string
  
  // Local Auth Status
  localAuthEnabled: boolean
  setLocalAuthEnabled: (enabled: boolean) => void
  
  // Theme (light or dark only, system preference detected on first load)
  theme: "light" | "dark"
  setTheme: (theme: "light" | "dark") => void
  
  // Sidebar
  sidebarCollapsed: boolean
  setSidebarCollapsed: (collapsed: boolean) => void
  toggleSidebar: () => void
  
  // Auth Dialog
  authDialogOpen: boolean
  setAuthDialogOpen: (open: boolean) => void
  
  // Login Dialog
  loginDialogOpen: boolean
  setLoginDialogOpen: (open: boolean) => void
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set, get) => ({
      // Static API Token (legacy support)
      apiToken: "",
      setApiToken: (token) => set({ apiToken: token }),
      clearApiToken: () => set({ apiToken: "" }),
      
      // Local Auth (JWT tokens)
      authTokens: null,
      setAuthTokens: (tokens) => set({ authTokens: tokens }),
      clearAuthTokens: () => set({ authTokens: null }),
      isLoggedIn: () => {
        const tokens = get().authTokens
        if (!tokens) return false
        // Check if access token is still valid (with 60s buffer)
        return tokens.expiresAt > Date.now() / 1000 + 60
      },
      getActiveToken: () => {
        const state = get()
        // If local auth is enabled and we have valid JWT tokens, use that
        if (state.localAuthEnabled && state.authTokens) {
          return state.authTokens.accessToken
        }
        // Fall back to static API token
        return state.apiToken
      },
      
      // Local Auth Status
      localAuthEnabled: false,
      setLocalAuthEnabled: (enabled) => set({ localAuthEnabled: enabled }),
      
      // Theme (detect system preference on first load)
      theme: typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light',
      setTheme: (theme) => set({ theme }),
      
      // Sidebar (collapsed by default)
      sidebarCollapsed: true,
      setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
      toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
      
      // Auth Dialog (WhatsApp auth, not persisted)
      authDialogOpen: false,
      setAuthDialogOpen: (open) => set({ authDialogOpen: open }),
      
      // Login Dialog (local auth, not persisted)
      loginDialogOpen: false,
      setLoginDialogOpen: (open) => set({ loginDialogOpen: open }),
    }),
    {
      name: "was-settings",
      partialize: (state) => ({
        apiToken: state.apiToken,
        authTokens: state.authTokens,
        localAuthEnabled: state.localAuthEnabled,
        theme: state.theme,
        sidebarCollapsed: state.sidebarCollapsed,
      }),
    }
  )
)
