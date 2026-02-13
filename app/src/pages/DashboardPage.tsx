import { HealthStatus } from "@/components/dashboard"
import { useQuery } from "@tanstack/react-query"
import { apiClient } from "@/lib/api"
import { Card, CardContent, CardHeader, CardTitle, Button } from "@/components/ui"
import { useNavigate } from "react-router-dom"
import { MessageSquare, Smartphone, ExternalLink } from "lucide-react"

export function DashboardPage() {
  const navigate = useNavigate()

  const authQuery = useQuery({
    queryKey: ["authStatus"],
    queryFn: () => apiClient.getAuthStatus(),
    refetchInterval: 10000,
  })

  return (
    <div className="p-6 space-y-6 overflow-auto h-full">
      <div>
        <h1 className="text-3xl font-bold">Dashboard</h1>
        <p className="text-muted-foreground">
          Monitor your WhatsApp Server status
        </p>
      </div>

      {/* Health Status Cards */}
      <HealthStatus />

      {/* Quick Actions */}
      <div className="grid gap-4 md:grid-cols-2">
        {!authQuery.data?.authenticated && (
          <Card className="border-dashed">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Smartphone className="h-5 w-5 text-yellow-500" />
                Link Your Device
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground mb-4">
                Scan the QR code or enter your phone number to connect WhatsApp
              </p>
              <Button variant="whatsapp" onClick={() => navigate("/auth")}>
                Link Device
              </Button>
            </CardContent>
          </Card>
        )}

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <MessageSquare className="h-5 w-5" />
              Start Messaging
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground mb-4">
              Send and receive messages through the chat interface
            </p>
            <Button onClick={() => navigate("/chat")}>
              Open Chats
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <ExternalLink className="h-5 w-5" />
              API Documentation
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground mb-4">
              Explore the REST API with interactive Swagger documentation
            </p>
            <Button variant="outline" asChild>
              <a href="/swagger-ui/" target="_blank" rel="noopener noreferrer">
                Open Swagger UI
              </a>
            </Button>
          </CardContent>
        </Card>
      </div>

      {/* Connected User Info */}
      {authQuery.data?.authenticated && (
        <Card className="border-whatsapp/50 bg-whatsapp/5">
          <CardHeader>
            <CardTitle className="text-whatsapp-dark">Connected Account</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-4">
              <div className="w-16 h-16 rounded-full bg-whatsapp/20 flex items-center justify-center">
                <Smartphone className="h-8 w-8 text-whatsapp" />
              </div>
              <div>
                <p className="font-semibold text-lg">
                  {authQuery.data.phone_number || "WhatsApp User"}
                </p>
                <p className="text-muted-foreground">
                  Device linked and ready
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
