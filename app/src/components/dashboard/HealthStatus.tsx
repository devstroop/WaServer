import { useQuery } from "@tanstack/react-query"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Spinner,
} from "@/components/ui"
import { apiClient } from "@/lib/api"
import {
  Activity,
  CheckCircle,
  XCircle,
  Clock,
  Smartphone,
  Server,
} from "lucide-react"

export function HealthStatus() {
  const healthQuery = useQuery({
    queryKey: ["health"],
    queryFn: () => apiClient.getHealth(),
    refetchInterval: 10000, // Refresh every 10 seconds
  })

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400)
    const hours = Math.floor((seconds % 86400) / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)

    const parts = []
    if (days > 0) parts.push(`${days}d`)
    if (hours > 0) parts.push(`${hours}h`)
    if (minutes > 0) parts.push(`${minutes}m`)
    return parts.join(" ") || "< 1m"
  }

  if (healthQuery.isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center p-8">
          <Spinner size="lg" />
        </CardContent>
      </Card>
    )
  }

  if (healthQuery.error) {
    return (
      <Card className="border-destructive">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-destructive">
            <XCircle className="h-5 w-5" />
            Server Unreachable
          </CardTitle>
          <CardDescription>
            Unable to connect to the WAS server
          </CardDescription>
        </CardHeader>
      </Card>
    )
  }

  const health = healthQuery.data!

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      {/* Server Status */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Server Status</CardTitle>
          <Server className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            {health.status === "healthy" ? (
              <>
                <CheckCircle className="h-5 w-5 text-whatsapp" />
                <span className="text-2xl font-bold text-whatsapp">Healthy</span>
              </>
            ) : (
              <>
                <XCircle className="h-5 w-5 text-destructive" />
                <span className="text-2xl font-bold text-destructive">Unhealthy</span>
              </>
            )}
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            Version: {health.version}
          </p>
        </CardContent>
      </Card>

      {/* Uptime */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Uptime</CardTitle>
          <Clock className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {formatUptime(health.uptime_seconds)}
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            {health.uptime_seconds.toLocaleString()} seconds
          </p>
        </CardContent>
      </Card>

      {/* Browser Status */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Browser</CardTitle>
          <Activity className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            {health.services?.whatsapp?.status === "healthy" ? (
              <>
                <CheckCircle className="h-5 w-5 text-whatsapp" />
                <span className="text-2xl font-bold text-whatsapp">Ready</span>
              </>
            ) : (
              <>
                <XCircle className="h-5 w-5 text-destructive" />
                <span className="text-2xl font-bold text-destructive">Not Ready</span>
              </>
            )}
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            Playwright browser instance
          </p>
        </CardContent>
      </Card>

      {/* WhatsApp Auth */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">WhatsApp</CardTitle>
          <Smartphone className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            {health.whatsapp_connection_status === "connected" ? (
              <>
                <CheckCircle className="h-5 w-5 text-whatsapp" />
                <span className="text-2xl font-bold text-whatsapp">Connected</span>
              </>
            ) : (
              <>
                <XCircle className="h-5 w-5 text-yellow-500" />
                <span className="text-2xl font-bold text-yellow-500">Not Linked</span>
              </>
            )}
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            {health.whatsapp_connection_status === "connected" ? "Device linked" : "Scan QR to connect"}
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
