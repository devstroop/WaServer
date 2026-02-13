import { useState } from "react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
  Button,
  Label,
  toast,
} from "@/components/ui"
import { useSettingsStore } from "@/store"
import { Key, Save, Moon, Sun } from "lucide-react"

export function Settings() {
  const { apiToken, setApiToken, theme, setTheme } = useSettingsStore()
  const [tokenInput, setTokenInput] = useState(apiToken)

  const handleSaveToken = () => {
    setApiToken(tokenInput)
    toast({
      title: "Settings saved",
      description: "API token has been updated",
      variant: "success",
    })
  }

  const handleThemeChange = (newTheme: "light" | "dark") => {
    setTheme(newTheme)
    
    // Apply theme to document
    const root = window.document.documentElement
    root.classList.remove("light", "dark")
    root.classList.add(newTheme)
  }

  return (
    <div className="space-y-6 max-w-2xl">
      {/* API Token */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Key className="h-5 w-5" />
            API Authentication
          </CardTitle>
          <CardDescription>
            Configure your API token for authenticating requests
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="apiToken">API Token</Label>
            <Input
              id="apiToken"
              type="password"
              placeholder="Enter your API token"
              value={tokenInput}
              onChange={(e) => setTokenInput(e.target.value)}
            />
            <p className="text-xs text-muted-foreground">
              This token is stored locally and used for all API requests
            </p>
          </div>
          <Button onClick={handleSaveToken} variant="whatsapp">
            <Save className="h-4 w-4 mr-2" />
            Save Token
          </Button>
        </CardContent>
      </Card>

      {/* Theme */}
      <Card>
        <CardHeader>
          <CardTitle>Appearance</CardTitle>
          <CardDescription>
            Customize the look and feel of the application
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            <Label>Theme</Label>
            <div className="flex gap-2">
              <Button
                variant={theme === "light" ? "default" : "outline"}
                size="sm"
                onClick={() => handleThemeChange("light")}
              >
                <Sun className="h-4 w-4 mr-2" />
                Light
              </Button>
              <Button
                variant={theme === "dark" ? "default" : "outline"}
                size="sm"
                onClick={() => handleThemeChange("dark")}
              >
                <Moon className="h-4 w-4 mr-2" />
                Dark
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* API Info */}
      <Card>
        <CardHeader>
          <CardTitle>API Documentation</CardTitle>
          <CardDescription>
            Resources for integrating with the WAS API
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          <p className="text-sm">
            <strong>Swagger UI:</strong>{" "}
            <a
              href="/swagger-ui/"
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary hover:underline"
            >
              /swagger-ui/
            </a>
          </p>
          <p className="text-sm">
            <strong>OpenAPI Spec:</strong>{" "}
            <a
              href="/api-docs/openapi.json"
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary hover:underline"
            >
              /api-docs/openapi.json
            </a>
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
