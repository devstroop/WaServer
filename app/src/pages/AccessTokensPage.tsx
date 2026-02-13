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
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui"
import { useSettingsStore } from "@/store"
import {
  Key,
  Copy,
  Check,
  Eye,
  EyeOff,
  Shield,
  Clock,
  AlertTriangle,
} from "lucide-react"

export function AccessTokensPage() {
  const { apiToken, setApiToken, authTokens, localAuthEnabled } = useSettingsStore()
  const [showToken, setShowToken] = useState(false)
  const [copied, setCopied] = useState(false)
  const [newToken, setNewToken] = useState("")

  const handleCopyToken = async () => {
    const tokenToCopy = localAuthEnabled && authTokens ? authTokens.accessToken : apiToken
    if (tokenToCopy) {
      await navigator.clipboard.writeText(tokenToCopy)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    }
  }

  const handleSaveStaticToken = () => {
    if (newToken.trim()) {
      setApiToken(newToken.trim())
      setNewToken("")
    }
  }

  const formatExpiry = (expiresAt: number) => {
    const now = Date.now() / 1000
    const diff = expiresAt - now
    if (diff <= 0) return "Expired"
    if (diff < 60) return `${Math.floor(diff)}s`
    if (diff < 3600) return `${Math.floor(diff / 60)}m`
    if (diff < 86400) return `${Math.floor(diff / 3600)}h`
    return `${Math.floor(diff / 86400)}d`
  }

  const maskToken = (token: string) => {
    if (token.length <= 8) return "••••••••"
    return token.slice(0, 4) + "••••••••" + token.slice(-4)
  }

  return (
    <div className="container max-w-4xl py-8 space-y-6">
      <div className="space-y-2">
        <h1 className="text-3xl font-bold tracking-tight">Access Tokens</h1>
        <p className="text-muted-foreground">
          Manage API authentication tokens for accessing WAS endpoints
        </p>
      </div>

      {/* Current Active Token */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-primary" />
            <CardTitle>Active Token</CardTitle>
          </div>
          <CardDescription>
            Currently active token used for API authentication
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {localAuthEnabled && authTokens ? (
            <>
              <div className="flex items-center gap-2">
                <span className="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold bg-green-500 text-white">
                  JWT Token
                </span>
                <span className="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold border border-border">
                  User: {authTokens.username}
                </span>
              </div>
              <div className="flex items-center gap-2">
                <div className="flex-1 font-mono text-sm bg-muted p-3 rounded-md overflow-hidden">
                  {showToken ? authTokens.accessToken : maskToken(authTokens.accessToken)}
                </div>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="outline"
                      size="icon"
                      onClick={() => setShowToken(!showToken)}
                    >
                      {showToken ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>{showToken ? "Hide" : "Show"} token</TooltipContent>
                </Tooltip>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="outline"
                      size="icon"
                      onClick={handleCopyToken}
                    >
                      {copied ? <Check className="h-4 w-4 text-green-500" /> : <Copy className="h-4 w-4" />}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>{copied ? "Copied!" : "Copy token"}</TooltipContent>
                </Tooltip>
              </div>
              <div className="flex items-center gap-4 text-sm text-muted-foreground">
                <div className="flex items-center gap-1">
                  <Clock className="h-4 w-4" />
                  <span>Expires in: {formatExpiry(authTokens.expiresAt)}</span>
                </div>
              </div>
            </>
          ) : apiToken ? (
            <>
              <div className="flex items-center gap-2">
                <span className="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold bg-secondary text-secondary-foreground">
                  Static API Token
                </span>
              </div>
              <div className="flex items-center gap-2">
                <div className="flex-1 font-mono text-sm bg-muted p-3 rounded-md overflow-hidden">
                  {showToken ? apiToken : maskToken(apiToken)}
                </div>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="outline"
                      size="icon"
                      onClick={() => setShowToken(!showToken)}
                    >
                      {showToken ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>{showToken ? "Hide" : "Show"} token</TooltipContent>
                </Tooltip>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="outline"
                      size="icon"
                      onClick={handleCopyToken}
                    >
                      {copied ? <Check className="h-4 w-4 text-green-500" /> : <Copy className="h-4 w-4" />}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>{copied ? "Copied!" : "Copy token"}</TooltipContent>
                </Tooltip>
              </div>
            </>
          ) : (
            <div className="flex items-center gap-2 text-muted-foreground">
              <AlertTriangle className="h-4 w-4 text-yellow-500" />
              <span>No token configured. API requests may fail.</span>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Static API Token Configuration */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Key className="h-5 w-5 text-primary" />
            <CardTitle>Static API Token</CardTitle>
          </div>
          <CardDescription>
            Configure a static API token for authentication. This is used when local auth is disabled.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="static-token">API Token</Label>
            <div className="flex gap-2">
              <Input
                id="static-token"
                type="text"
                placeholder="Enter your API token"
                value={newToken}
                onChange={(e) => setNewToken(e.target.value)}
                className="font-mono"
              />
              <Button onClick={handleSaveStaticToken} disabled={!newToken.trim()}>
                Save
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              This token should match the <code className="bg-muted px-1 rounded">api_token</code> value 
              in your server's <code className="bg-muted px-1 rounded">config/app.toml</code>
            </p>
          </div>

          {apiToken && (
            <div className="pt-4 border-t">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium">Current Static Token</p>
                  <p className="text-xs text-muted-foreground font-mono">
                    {maskToken(apiToken)}
                  </p>
                </div>
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={() => setApiToken("")}
                >
                  Remove
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* API Usage Examples */}
      <Card>
        <CardHeader>
          <CardTitle>API Usage</CardTitle>
          <CardDescription>
            How to use your access token with the API
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>cURL Example</Label>
            <pre className="bg-muted p-4 rounded-md text-sm overflow-x-auto">
              <code>{`curl -X GET "http://localhost:3000/api/v1/chats" \\
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \\
  -H "Content-Type: application/json"`}</code>
            </pre>
          </div>
          
          <div className="space-y-2">
            <Label>JavaScript/Fetch Example</Label>
            <pre className="bg-muted p-4 rounded-md text-sm overflow-x-auto">
              <code>{`fetch('/api/v1/chats', {
  headers: {
    'Authorization': 'Bearer YOUR_TOKEN_HERE',
    'Content-Type': 'application/json'
  }
})
.then(res => res.json())
.then(data => console.log(data))`}</code>
            </pre>
          </div>
        </CardContent>
      </Card>

      {/* Security Notes */}
      <Card className="border-yellow-500/50">
        <CardHeader>
          <div className="flex items-center gap-2 text-yellow-500">
            <AlertTriangle className="h-5 w-5" />
            <CardTitle className="text-yellow-500">Security Notes</CardTitle>
          </div>
        </CardHeader>
        <CardContent>
          <ul className="list-disc list-inside space-y-2 text-sm text-muted-foreground">
            <li>Never share your access tokens publicly or commit them to version control</li>
            <li>JWT tokens expire automatically for security - refresh them as needed</li>
            <li>Static API tokens should be rotated periodically</li>
            <li>Use HTTPS in production to protect tokens in transit</li>
            <li>Store tokens securely - they are saved in browser localStorage</li>
          </ul>
        </CardContent>
      </Card>
    </div>
  )
}
