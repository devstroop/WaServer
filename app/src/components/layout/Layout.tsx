import { Outlet } from "react-router-dom"
import { Sidebar } from "./Sidebar"
import { Header } from "./Header"
import { Toaster, TooltipProvider } from "@/components/ui"
import { useSettingsStore } from "@/store"
import { TokenInput } from "./TokenInput"

export function Layout() {
  const { apiToken, authTokens, isLoggedIn } = useSettingsStore()

  // Check if user is authenticated via either method
  const hasStaticToken = !!apiToken
  const hasValidJwt = authTokens && isLoggedIn()
  const isAuthenticated = hasStaticToken || hasValidJwt

  // Show login/token input if not authenticated
  if (!isAuthenticated) {
    return (
      <div className="flex h-screen bg-background items-center justify-center p-4">
        <TokenInput />
        <Toaster />
      </div>
    )
  }

  return (
    <TooltipProvider>
      <div className="flex h-screen bg-background">
        <Sidebar />
        <div className="flex-1 flex flex-col overflow-hidden">
          <Header />
          <main className="flex-1 overflow-auto">
            <Outlet />
          </main>
        </div>
        <Toaster />
      </div>
    </TooltipProvider>
  )
}
