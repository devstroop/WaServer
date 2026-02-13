import { useState } from "react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Button,
  Input,
  Label,
  Switch,
  toast,
} from "@/components/ui"
import {
  Webhook,
  Plus,
  Trash2,
  ExternalLink,
  AlertCircle,
  CheckCircle2,
  Info,
} from "lucide-react"

interface WebhookEndpoint {
  id: string
  url: string
  secret: string
  enabled: boolean
  lastStatus?: "success" | "error" | "pending"
  lastTriggered?: string
}

export function WebhooksPage() {
  // Note: In a real implementation, this would connect to the backend config
  // For now, this serves as a UI reference showing webhook configuration
  const [webhooksEnabled, setWebhooksEnabled] = useState(false)
  const [endpoints, setEndpoints] = useState<WebhookEndpoint[]>([])
  const [newUrl, setNewUrl] = useState("")
  const [newSecret, setNewSecret] = useState("")

  const addEndpoint = () => {
    if (!newUrl.trim()) {
      toast({
        title: "URL required",
        description: "Please enter a webhook URL",
        variant: "destructive",
      })
      return
    }

    try {
      new URL(newUrl)
    } catch {
      toast({
        title: "Invalid URL",
        description: "Please enter a valid URL starting with http:// or https://",
        variant: "destructive",
      })
      return
    }

    const newEndpoint: WebhookEndpoint = {
      id: crypto.randomUUID(),
      url: newUrl.trim(),
      secret: newSecret.trim(),
      enabled: true,
      lastStatus: "pending",
    }

    setEndpoints([...endpoints, newEndpoint])
    setNewUrl("")
    setNewSecret("")

    toast({
      title: "Webhook added",
      description: "Remember to update your app.toml configuration",
    })
  }

  const removeEndpoint = (id: string) => {
    setEndpoints(endpoints.filter((e) => e.id !== id))
  }

  const toggleEndpoint = (id: string) => {
    setEndpoints(
      endpoints.map((e) =>
        e.id === id ? { ...e, enabled: !e.enabled } : e
      )
    )
  }

  return (
    <div className="p-6 overflow-auto h-full">
      <div className="mb-6">
        <h1 className="text-3xl font-bold">Webhooks</h1>
        <p className="text-muted-foreground">
          Configure HTTP callbacks for message events
        </p>
      </div>

      <div className="space-y-6 max-w-4xl">
        {/* Info Card */}
        <Card className="border-blue-200 bg-blue-50 dark:border-blue-900 dark:bg-blue-950/30">
          <CardContent className="pt-6">
            <div className="flex gap-3">
              <Info className="h-5 w-5 text-blue-600 dark:text-blue-400 flex-shrink-0 mt-0.5" />
              <div className="text-sm text-blue-800 dark:text-blue-200">
                <p className="font-medium mb-1">Configuration via app.toml</p>
                <p className="text-blue-700 dark:text-blue-300">
                  Webhook endpoints are configured in the server's{" "}
                  <code className="px-1 py-0.5 bg-blue-100 dark:bg-blue-900 rounded">
                    config/app.toml
                  </code>{" "}
                  file. This page helps you visualize and plan your webhook setup.
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Global Enable */}
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-primary/10">
                  <Webhook className="h-5 w-5 text-primary" />
                </div>
                <div>
                  <CardTitle>Webhook Notifications</CardTitle>
                  <CardDescription>
                    Send HTTP callbacks when messages are received
                  </CardDescription>
                </div>
              </div>
              <Switch
                checked={webhooksEnabled}
                onCheckedChange={setWebhooksEnabled}
              />
            </div>
          </CardHeader>
          {webhooksEnabled && (
            <CardContent className="border-t pt-6">
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">Timeout:</span>
                  <span className="ml-2 font-medium">5000ms</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Retry Count:</span>
                  <span className="ml-2 font-medium">3</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Retry Delay:</span>
                  <span className="ml-2 font-medium">1000ms</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Event Type:</span>
                  <span className="ml-2 font-medium">message.received</span>
                </div>
              </div>
            </CardContent>
          )}
        </Card>

        {/* Endpoints List */}
        {webhooksEnabled && (
          <>
            <Card>
              <CardHeader>
                <CardTitle>Webhook Endpoints</CardTitle>
                <CardDescription>
                  URLs that will receive POST requests when messages arrive
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                {/* Add New Endpoint */}
                <div className="p-4 border rounded-lg bg-muted/30 space-y-4">
                  <div className="grid gap-4">
                    <div className="space-y-2">
                      <Label htmlFor="webhook-url">Endpoint URL</Label>
                      <Input
                        id="webhook-url"
                        placeholder="https://your-server.com/webhook/whatsapp"
                        value={newUrl}
                        onChange={(e) => setNewUrl(e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="webhook-secret">
                        HMAC Secret{" "}
                        <span className="text-muted-foreground font-normal">
                          (optional)
                        </span>
                      </Label>
                      <Input
                        id="webhook-secret"
                        placeholder="your-hmac-secret-for-signature-verification"
                        type="password"
                        value={newSecret}
                        onChange={(e) => setNewSecret(e.target.value)}
                      />
                      <p className="text-xs text-muted-foreground">
                        Used to sign webhook payloads with X-Webhook-Signature
                        header
                      </p>
                    </div>
                  </div>
                  <Button onClick={addEndpoint} className="w-full">
                    <Plus className="h-4 w-4 mr-2" />
                    Add Endpoint
                  </Button>
                </div>

                {/* Endpoint List */}
                {endpoints.length === 0 ? (
                  <div className="text-center py-8 text-muted-foreground">
                    <Webhook className="h-12 w-12 mx-auto mb-3 opacity-30" />
                    <p>No webhook endpoints configured</p>
                    <p className="text-sm">Add an endpoint above to get started</p>
                  </div>
                ) : (
                  <div className="space-y-3">
                    {endpoints.map((endpoint) => (
                      <div
                        key={endpoint.id}
                        className="flex items-center gap-3 p-3 border rounded-lg"
                      >
                        {/* Status Indicator */}
                        <div className="flex-shrink-0">
                          {endpoint.lastStatus === "success" && (
                            <CheckCircle2 className="h-5 w-5 text-green-500" />
                          )}
                          {endpoint.lastStatus === "error" && (
                            <AlertCircle className="h-5 w-5 text-red-500" />
                          )}
                          {endpoint.lastStatus === "pending" && (
                            <div className="h-5 w-5 rounded-full border-2 border-muted-foreground/30" />
                          )}
                        </div>

                        {/* URL */}
                        <div className="flex-1 min-w-0">
                          <p className="font-mono text-sm truncate">
                            {endpoint.url}
                          </p>
                          {endpoint.secret && (
                            <p className="text-xs text-muted-foreground">
                              HMAC signing enabled
                            </p>
                          )}
                        </div>

                        {/* Toggle */}
                        <Switch
                          checked={endpoint.enabled}
                          onCheckedChange={() => toggleEndpoint(endpoint.id)}
                        />

                        {/* Actions */}
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() =>
                            window.open(endpoint.url, "_blank")
                          }
                        >
                          <ExternalLink className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="text-destructive hover:text-destructive"
                          onClick={() => removeEndpoint(endpoint.id)}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>

            {/* Config Preview */}
            <Card>
              <CardHeader>
                <CardTitle>Configuration Preview</CardTitle>
                <CardDescription>
                  Copy this to your config/app.toml file
                </CardDescription>
              </CardHeader>
              <CardContent>
                <pre className="p-4 bg-muted rounded-lg overflow-x-auto text-sm font-mono">
                  {`[webhooks]
enabled = ${webhooksEnabled}
timeout_ms = 5000
retry_count = 3
retry_delay_ms = 1000
${endpoints
  .filter((e) => e.enabled)
  .map(
    (e) => `
[[webhooks.endpoints]]
url = "${e.url}"${e.secret ? `\nsecret = "${e.secret}"` : ""}`
  )
  .join("\n")}`}
                </pre>
                <Button
                  variant="outline"
                  className="mt-4"
                  onClick={() => {
                    navigator.clipboard.writeText(
                      `[webhooks]\nenabled = ${webhooksEnabled}\ntimeout_ms = 5000\nretry_count = 3\nretry_delay_ms = 1000\n${endpoints
                        .filter((e) => e.enabled)
                        .map(
                          (e) =>
                            `\n[[webhooks.endpoints]]\nurl = "${e.url}"${
                              e.secret ? `\nsecret = "${e.secret}"` : ""
                            }`
                        )
                        .join("\n")}`
                    )
                    toast({
                      title: "Copied!",
                      description: "Configuration copied to clipboard",
                    })
                  }}
                >
                  Copy to Clipboard
                </Button>
              </CardContent>
            </Card>
          </>
        )}

        {/* Payload Example */}
        <Card>
          <CardHeader>
            <CardTitle>Webhook Payload</CardTitle>
            <CardDescription>
              Example payload sent to your endpoints
            </CardDescription>
          </CardHeader>
          <CardContent>
            <pre className="p-4 bg-muted rounded-lg overflow-x-auto text-sm font-mono">
              {`{
  "event": "message.received",
  "timestamp": "2026-02-12T10:30:00Z",
  "data": {
    "message_id": "ABC123...",
    "from": "+1234567890",
    "to": "+0987654321",
    "text": "Hello, World!",
    "type": "text",
    "timestamp": "2026-02-12T10:30:00Z"
  }
}`}
            </pre>
            <div className="mt-4 space-y-2 text-sm text-muted-foreground">
              <p>
                <strong>Headers sent:</strong>
              </p>
              <ul className="list-disc list-inside space-y-1 ml-2">
                <li>
                  <code>Content-Type: application/json</code>
                </li>
                <li>
                  <code>X-Webhook-Event: message.received</code>
                </li>
                <li>
                  <code>X-Webhook-Timestamp: &lt;unix_timestamp&gt;</code>
                </li>
                <li>
                  <code>X-Webhook-Signature: &lt;hmac_sha256&gt;</code> (if secret
                  configured)
                </li>
              </ul>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
