import { AuthPanel } from "@/components/auth"
import { useNavigate } from "react-router-dom"

export function AuthPage() {
  const navigate = useNavigate()

  return (
    <div className="flex items-center justify-center h-full p-6 bg-muted/30">
      <AuthPanel onAuthenticated={() => navigate("/")} />
    </div>
  )
}
