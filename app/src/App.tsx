import { BrowserRouter, Routes, Route } from "react-router-dom"
import { Layout } from "@/components/layout"
import { DashboardPage, ChatPage, AuthPage, SettingsPage, WebhooksPage, AccessTokensPage } from "@/pages"
import { useEffect } from "react"
import { useSettingsStore } from "@/store"

function App() {
  const { theme } = useSettingsStore()

  // Apply theme on mount and when it changes
  useEffect(() => {
    const root = window.document.documentElement
    root.classList.remove("light", "dark")
    root.classList.add(theme)
  }, [theme])

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<DashboardPage />} />
          <Route path="chat" element={<ChatPage />} />
          <Route path="auth" element={<AuthPage />} />
          <Route path="webhooks" element={<WebhooksPage />} />
          <Route path="access-tokens" element={<AccessTokensPage />} />
          <Route path="settings" element={<SettingsPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}

export default App
