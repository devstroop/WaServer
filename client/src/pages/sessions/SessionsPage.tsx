import { useState } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { 
  RefreshCw, 
  Smartphone, 
  Monitor,
  Unplug,
  RotateCw,
  Zap,
  MessageSquare
} from 'lucide-react';
import { ContentContainer } from '@/components/layout/ContentContainer';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Tooltip,
} from 'recharts';

// Mock active sessions
const activeSessions = [
  {
    id: 1,
    name: 'Admin',
    device: 'iPhone 13',
    phone: '+1 (555) 012-3456',
    battery: 85,
    batteryStatus: 'charged',
    sessionExpiry: 'Oct 24, 2024',
  },
  {
    id: 2,
    name: 'Support',
    device: 'Desktop',
    phone: '+1 (555) 012-8899',
    battery: null,
    batteryStatus: 'plugged',
    sessionExpiry: 'Oct 28, 2024',
  },
  {
    id: 3,
    name: 'Sales',
    device: 'Android',
    phone: '+1 (555) 012-4411',
    battery: 18,
    batteryStatus: 'low',
    sessionExpiry: 'Oct 22, 2024',
  },
  {
    id: 4,
    name: 'Warehouse',
    device: 'iPad',
    phone: '+1 (555) 012-9900',
    battery: 65,
    batteryStatus: 'charged',
    sessionExpiry: 'Nov 01, 2024',
  },
];

// Mock throughput data
const throughputData = [
  { day: 'MON', value: 120 },
  { day: 'TUE', value: 180 },
  { day: 'WED', value: 240 },
  { day: 'THU', value: 280 },
  { day: 'FRI', value: 200 },
];

export function SessionsPage() {
  const [qrKey, setQrKey] = useState(0);
  const [isRefreshing, setIsRefreshing] = useState(false);

  const handleRefreshQR = () => {
    setIsRefreshing(true);
    setTimeout(() => {
      setQrKey((prev) => prev + 1);
      setIsRefreshing(false);
    }, 1000);
  };

  const getBatteryDisplay = (battery: number | null, status: string) => {
    if (status === 'plugged') {
      return (
        <div className="flex items-center gap-1 text-muted-foreground">
          <Zap className="h-3 w-3" />
          <span className="text-sm">Plugged</span>
        </div>
      );
    }
    if (battery === null) return '-';

    const barColor = battery >= 60 ? 'bg-success' : battery >= 30 ? 'bg-warning' : 'bg-destructive';
    
    return (
      <div className="flex items-center gap-2">
        <div className="w-10 h-3 bg-muted rounded-sm overflow-hidden border border-border relative">
          <div 
            className={`h-full ${barColor}`}
            style={{ width: `${battery}%` }}
          />
        </div>
        <span className="text-sm">{battery}%</span>
      </div>
    );
  };

  return (
    <ContentContainer>
      <div className="grid gap-6 lg:grid-cols-4">
        {/* Left Panel - QR Code */}
        <div className="space-y-6">
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-base">
                <Smartphone className="h-4 w-4 text-primary" />
                Connect New Device
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              {/* QR Code */}
              <div className="w-full bg-card rounded-lg flex items-center justify-center p-3 border border-dashed border-border">
                <div className="bg-white p-3 rounded-lg shadow-lg">
                  {isRefreshing ? (
                    <div className="text-center w-36 h-36 flex flex-col items-center justify-center">
                      <RefreshCw className="h-8 w-8 mx-auto animate-spin text-gray-400" />
                      <p className="text-xs text-gray-500 mt-2">Refreshing...</p>
                    </div>
                  ) : (
                    <QRCodeSVG
                      key={qrKey}
                      value={`whatsapp-connect-${qrKey}-${Date.now()}`}
                      size={144}
                      level="M"
                      includeMargin={false}
                      bgColor="#ffffff"
                      fgColor="#000000"
                    />
                  )}
                </div>
              </div>

              {/* Instructions */}
              <div className="text-center space-y-1">
                <h3 className="font-semibold text-sm">Scan QR Code</h3>
                <p className="text-xs text-muted-foreground">
                  WhatsApp → Settings → Linked Devices → Link a Device
                </p>
              </div>

              {/* Refresh Button */}
              <Button 
                variant="outline" 
                className="w-full" 
                onClick={handleRefreshQR}
                disabled={isRefreshing}
              >
                <RefreshCw className={`h-4 w-4 mr-2 ${isRefreshing ? 'animate-spin' : ''}`} />
                Refresh QR Code
              </Button>
            </CardContent>
          </Card>

          {/* Message Throughput */}
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="flex items-center gap-2 text-sm">
                <MessageSquare className="h-4 w-4 text-primary" />
                Throughput
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="h-28">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={throughputData}>
                    <XAxis 
                      dataKey="day" 
                      axisLine={false}
                      tickLine={false}
                      tick={{ fill: 'hsl(var(--muted-foreground))', fontSize: 12 }}
                    />
                    <YAxis hide />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: 'hsl(var(--popover))',
                        border: '1px solid hsl(var(--border))',
                        borderRadius: '8px',
                        color: 'hsl(var(--popover-foreground))',
                      }}
                      formatter={(value) => [`${value} messages`, 'Sent']}
                    />
                    <Bar 
                      dataKey="value" 
                      fill="hsl(var(--primary))"
                      radius={[4, 4, 0, 0]}
                    />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Right Panel - Active Sessions */}
        <div className="lg:col-span-3 space-y-6">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center gap-2 text-lg">
                  <Monitor className="h-5 w-5 text-primary" />
                  Active Sessions
                </CardTitle>
                <span className="text-sm text-muted-foreground">
                  {activeSessions.length} Devices Connected
                </span>
              </div>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-border">
                      <th className="text-left py-3 px-2 text-xs font-medium text-muted-foreground uppercase">Device Name</th>
                      <th className="text-left py-3 px-2 text-xs font-medium text-muted-foreground uppercase">Phone Number</th>
                      <th className="text-left py-3 px-2 text-xs font-medium text-muted-foreground uppercase">Battery</th>
                      <th className="text-left py-3 px-2 text-xs font-medium text-muted-foreground uppercase">Session Expiry</th>
                      <th className="text-left py-3 px-2 text-xs font-medium text-muted-foreground uppercase">Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {activeSessions.map((session) => (
                      <tr key={session.id} className="border-b border-border/50 hover:bg-muted/50">
                        <td className="py-4 px-2">
                          <div className="flex items-center gap-3">
                            <div className="h-8 w-8 rounded-lg bg-muted flex items-center justify-center">
                              {session.device.includes('Desktop') ? (
                                <Monitor className="h-4 w-4 text-muted-foreground" />
                              ) : (
                                <Smartphone className="h-4 w-4 text-muted-foreground" />
                              )}
                            </div>
                            <div>
                              <p className="font-medium">{session.name}</p>
                              <p className="text-xs text-muted-foreground">{session.device}</p>
                            </div>
                          </div>
                        </td>
                        <td className="py-4 px-2 text-sm">{session.phone}</td>
                        <td className="py-4 px-2">
                          {getBatteryDisplay(session.battery, session.batteryStatus)}
                        </td>
                        <td className="py-4 px-2 text-sm">{session.sessionExpiry}</td>
                        <td className="py-4 px-2">
                          <Button variant="destructive" size="sm">
                            Logout
                          </Button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              <button className="w-full text-center text-primary text-sm hover:underline py-4">
                View Historical Sessions
              </button>
            </CardContent>
          </Card>

          {/* Quick Actions */}
          <div className="grid gap-4 md:grid-cols-2">
            <Card className="hover:border-primary/50 transition-colors cursor-pointer">
              <CardContent className="p-6">
                <div className="flex items-center gap-4">
                  <div className="h-12 w-12 rounded-lg bg-primary/10 flex items-center justify-center">
                    <RotateCw className="h-6 w-6 text-primary" />
                  </div>
                  <div>
                    <h3 className="font-semibold">Force Resync</h3>
                    <p className="text-sm text-muted-foreground">Update all session states</p>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card className="hover:border-destructive/50 transition-colors cursor-pointer">
              <CardContent className="p-6">
                <div className="flex items-center gap-4">
                  <div className="h-12 w-12 rounded-lg bg-destructive/10 flex items-center justify-center">
                    <Unplug className="h-6 w-6 text-destructive" />
                  </div>
                  <div>
                    <h3 className="font-semibold">Global Logout</h3>
                    <p className="text-sm text-muted-foreground">Disconnect all devices</p>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </ContentContainer>
  );
}
