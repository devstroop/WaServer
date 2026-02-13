import { useState } from "react"
import { useQuery, useMutation } from "@tanstack/react-query"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Button,
  Input,
  Label,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Spinner,
  toast,
} from "@/components/ui"
import { apiClient } from "@/lib/api"
import { useSettingsStore } from "@/store"
import { Smartphone, QrCode, RefreshCw, Key, CheckCircle2, Loader2 } from "lucide-react"
import { cn } from "@/lib/utils"

interface AuthPanelProps {
  onAuthenticated?: () => void
  compact?: boolean
}

export function AuthPanel({ onAuthenticated, compact = false }: AuthPanelProps) {
  const [phoneNumber, setPhoneNumber] = useState("")
  const [pairingCode, setPairingCode] = useState<string | null>(null)
  const [tokenInput, setTokenInput] = useState("")
  const { apiToken, setApiToken } = useSettingsStore()

  // Auth status polling
  const statusQuery = useQuery({
    queryKey: ["authStatus"],
    queryFn: () => apiClient.getAuthStatus(),
    refetchInterval: 3000, // Check every 3 seconds
    enabled: !!apiToken, // Only fetch if token is set
  })

  // Determine auth state
  const isChecking = statusQuery.data?.status === "checking" || statusQuery.isLoading
  const isAuthenticated = statusQuery.data?.authenticated === true
  const needsAuth = statusQuery.data?.status === "not_authenticated"

  // QR Code query - only fetch when we definitely need auth
  const qrQuery = useQuery({
    queryKey: ["qrCode"],
    queryFn: () => apiClient.getQRCode(),
    refetchInterval: 30000, // Refresh every 30 seconds
    enabled: !!apiToken && needsAuth, // Only fetch if token is set AND auth is needed
  })

  // Phone pairing mutation
  const phonePairMutation = useMutation({
    mutationFn: (phone: string) => apiClient.requestPhonePairing(phone),
    onSuccess: (data) => {
      setPairingCode(data.pairing_code)
      toast({
        title: "Pairing code generated",
        description: `Enter code ${data.pairing_code} on your phone`,
        variant: "success",
      })
    },
    onError: (error: Error) => {
      toast({
        title: "Failed to generate pairing code",
        description: error.message,
        variant: "destructive",
      })
    },
  })

  // Check if authenticated
  if (isAuthenticated) {
    onAuthenticated?.()
  }

  const handlePhonePair = (e: React.FormEvent) => {
    e.preventDefault()
    if (phoneNumber.trim()) {
      phonePairMutation.mutate(phoneNumber.trim())
    }
  }

  const handleTokenSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (tokenInput.trim()) {
      setApiToken(tokenInput.trim())
      toast({
        title: "API Token saved",
        description: "You can now authenticate with WhatsApp",
        variant: "success",
      })
    }
  }

  // Show token input if not configured
  if (!apiToken) {
    return (
      <Card className={cn("w-full max-w-md mx-auto", compact && "border-0 shadow-none")}>
        <CardHeader className="text-center">
          <CardTitle className="text-2xl flex items-center justify-center gap-2">
            <Key className="h-6 w-6 text-whatsapp" />
            API Authentication
          </CardTitle>
          <CardDescription>
            Enter your API token to connect to the server
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleTokenSubmit} className="space-y-4">
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
                Find your token in <code className="bg-muted px-1 rounded">config/app.toml</code> under <code className="bg-muted px-1 rounded">[auth]</code> section
              </p>
            </div>
            <Button
              type="submit"
              variant="whatsapp"
              className="w-full"
              disabled={!tokenInput.trim()}
            >
              Connect
            </Button>
          </form>
        </CardContent>
      </Card>
    )
  }

  // Show checking state while WhatsApp is loading
  if (isChecking && !needsAuth) {
    return (
      <div className={cn("w-full", compact ? "py-4" : "max-w-md mx-auto")}>
        {!compact && (
          <Card>
            <CardHeader className="text-center">
              <CardTitle className="text-2xl flex items-center justify-center gap-2">
                <Loader2 className="h-6 w-6 text-whatsapp animate-spin" />
                Connecting to WhatsApp
              </CardTitle>
              <CardDescription>
                Please wait while WhatsApp Web loads...
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex flex-col items-center gap-4 py-8">
                <div className="w-16 h-16 flex items-center justify-center rounded-full bg-whatsapp/10">
                  <Spinner size="lg" className="text-whatsapp" />
                </div>
                <p className="text-sm text-muted-foreground text-center">
                  Checking authentication status...
                </p>
              </div>
            </CardContent>
          </Card>
        )}
        {compact && (
          <div className="flex flex-col items-center gap-4 py-8">
            <div className="w-16 h-16 flex items-center justify-center rounded-full bg-whatsapp/10">
              <Spinner size="lg" className="text-whatsapp" />
            </div>
            <p className="text-sm text-muted-foreground text-center">
              Checking authentication status...
            </p>
          </div>
        )}
      </div>
    )
  }

  // Show connected state
  if (isAuthenticated) {
    return (
      <div className={cn("w-full", compact ? "py-4" : "max-w-md mx-auto")}>
        {!compact && (
          <Card>
            <CardHeader className="text-center">
              <CardTitle className="text-2xl flex items-center justify-center gap-2">
                <CheckCircle2 className="h-6 w-6 text-whatsapp" />
                Device Connected
              </CardTitle>
              <CardDescription>
                Your WhatsApp device is linked and ready
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex flex-col items-center gap-4 py-4">
                <div className="p-4 bg-whatsapp/10 rounded-lg text-center w-full">
                  <p className="text-whatsapp-dark font-medium">
                    ✓ Connected
                    {statusQuery.data?.phone_number && (
                      <span className="block text-sm text-muted-foreground mt-1">
                        {statusQuery.data.phone_number}
                      </span>
                    )}
                  </p>
                </div>
                <p className="text-sm text-muted-foreground text-center">
                  You can now send and receive messages
                </p>
              </div>
            </CardContent>
          </Card>
        )}
        {compact && (
          <div className="flex flex-col items-center gap-4 py-4">
            <div className="w-16 h-16 flex items-center justify-center rounded-full bg-whatsapp/10">
              <CheckCircle2 className="h-8 w-8 text-whatsapp" />
            </div>
            <p className="text-whatsapp-dark font-medium text-center">
              ✓ Device Connected
              {statusQuery.data?.phone_number && (
                <span className="block text-sm text-muted-foreground mt-1">
                  {statusQuery.data.phone_number}
                </span>
              )}
            </p>
          </div>
        )}
      </div>
    )
  }

  // QR size based on compact mode
  const qrSize = compact ? 200 : 256

  // Main auth UI
  const authContent = (
    <Tabs defaultValue="qr" className="w-full">
      <TabsList className="grid w-full grid-cols-2">
        <TabsTrigger value="qr" className="flex items-center gap-2">
          <QrCode className="h-4 w-4" />
          QR Code
        </TabsTrigger>
        <TabsTrigger value="phone" className="flex items-center gap-2">
          <Smartphone className="h-4 w-4" />
          Phone
        </TabsTrigger>
      </TabsList>

      <TabsContent value="qr" className="mt-4">
        <div className="flex flex-col items-center gap-4">
          {qrQuery.isLoading ? (
            <div 
              className="flex items-center justify-center bg-muted rounded-lg"
              style={{ width: qrSize, height: qrSize }}
            >
              <Spinner size="lg" />
            </div>
          ) : qrQuery.error ? (
            <div 
              className="flex flex-col items-center justify-center bg-muted rounded-lg gap-2"
              style={{ width: qrSize, height: qrSize }}
            >
              <p className="text-sm text-destructive">Failed to load QR code</p>
              <Button
                variant="outline"
                size="sm"
                onClick={() => qrQuery.refetch()}
              >
                <RefreshCw className="h-4 w-4 mr-2" />
                Retry
              </Button>
            </div>
          ) : qrQuery.data?.qrcode ? (
            <div className="qr-container bg-white p-2 rounded-lg shadow-sm">
              <img
                src={`data:image/png;base64,${qrQuery.data.qrcode}`}
                alt="WhatsApp QR Code"
                width={qrSize}
                height={qrSize}
                className="rounded"
              />
            </div>
          ) : (
            <div 
              className="flex items-center justify-center bg-muted rounded-lg"
              style={{ width: qrSize, height: qrSize }}
            >
              <p className="text-sm text-muted-foreground">No QR code available</p>
            </div>
          )}

          <div className={cn("text-center space-y-1", compact ? "text-xs" : "text-sm")}>
            <p className="text-muted-foreground">
              1. Open WhatsApp → Linked Devices
            </p>
            <p className="text-muted-foreground">
              2. Tap "Link a Device" → Scan QR
            </p>
          </div>

          <Button
            variant="outline"
            size={compact ? "sm" : "default"}
            onClick={() => qrQuery.refetch()}
            disabled={qrQuery.isFetching}
          >
            <RefreshCw className={cn("h-4 w-4 mr-2", qrQuery.isFetching && "animate-spin")} />
            Refresh
          </Button>
        </div>
      </TabsContent>

      <TabsContent value="phone" className="mt-4">
        <form onSubmit={handlePhonePair} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="phone">Phone Number</Label>
            <Input
              id="phone"
              type="tel"
              placeholder="+1234567890"
              value={phoneNumber}
              onChange={(e) => setPhoneNumber(e.target.value)}
              disabled={phonePairMutation.isPending}
              className={cn(compact && "h-10")}
            />
            <p className="text-xs text-muted-foreground">
              Include country code (e.g., +1 for US)
            </p>
          </div>

          <Button
            type="submit"
            variant="whatsapp"
            className="w-full"
            size={compact ? "default" : "lg"}
            disabled={!phoneNumber.trim() || phonePairMutation.isPending}
          >
            {phonePairMutation.isPending ? (
              <>
                <Spinner size="sm" className="mr-2 text-white" />
                Generating...
              </>
            ) : (
              "Get Pairing Code"
            )}
          </Button>

          {pairingCode && (
            <div className="p-4 bg-whatsapp/10 rounded-lg text-center animate-in fade-in-0 zoom-in-95">
              <p className="text-xs text-muted-foreground mb-2">
                Enter this code on your phone:
              </p>
              <p className="text-2xl font-mono font-bold text-whatsapp-dark tracking-wider">
                {pairingCode}
              </p>
            </div>
          )}

          <div className={cn("text-center space-y-1 pt-2", compact ? "text-xs" : "text-sm")}>
            <p className="text-muted-foreground">
              1. Open WhatsApp → Linked Devices
            </p>
            <p className="text-muted-foreground">
              2. Tap "Link with phone number"
            </p>
            <p className="text-muted-foreground">
              3. Enter the pairing code above
            </p>
          </div>
        </form>
      </TabsContent>
    </Tabs>
  )

  if (compact) {
    return authContent
  }

  return (
    <Card className="w-full max-w-md mx-auto">
      <CardHeader className="text-center">
        <CardTitle className="text-2xl flex items-center justify-center gap-2">
          <span className="text-whatsapp">●</span>
          WhatsApp Authentication
        </CardTitle>
        <CardDescription>
          Scan QR code or enter phone number to link your account
        </CardDescription>
      </CardHeader>
      <CardContent>
        {authContent}
      </CardContent>
    </Card>
  )
}
