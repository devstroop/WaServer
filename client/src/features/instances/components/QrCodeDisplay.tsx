import { useEffect, useState } from 'react';
import { Loader2, RefreshCw, CheckCircle } from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { useQrCode, useWhatsAppStatus } from '../hooks/useWhatsApp';
import { Skeleton } from '@/components/ui/Skeleton';

interface QrCodeDisplayProps {
  instanceId: string;
}

export function QrCodeDisplay({ instanceId }: QrCodeDisplayProps) {
  const { data: qrData, isLoading, refetch, isFetching } = useQrCode(instanceId);
  const { data: status } = useWhatsAppStatus(instanceId);
  const [timeLeft, setTimeLeft] = useState<number | null>(null);

  useEffect(() => {
    if (!qrData?.expires_at) return;
    const expiresAt = new Date(qrData.expires_at).getTime();
    const updateTimer = () => {
      const remaining = Math.max(0, Math.floor((expiresAt - Date.now()) / 1000));
      setTimeLeft(remaining);
      if (remaining === 0) refetch();
    };
    updateTimer();
    const interval = setInterval(updateTimer, 1000);
    return () => clearInterval(interval);
  }, [qrData?.expires_at, refetch]);

  if (status?.connected) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <CheckCircle className="h-5 w-5 text-green-500" />Connected
          </CardTitle>
          <CardDescription>WhatsApp is connected and ready to use.</CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-sm">Phone: {status.phone_number}</p>
          {status.battery_level !== null && (
            <p className="text-sm text-muted-foreground">
              Battery: {status.battery_level}% {status.is_plugged && '(charging)'}
            </p>
          )}
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Scan QR Code</CardTitle>
        <CardDescription>Open WhatsApp on your phone, go to Settings &gt; Linked Devices &gt; Link a Device</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col items-center">
        {isLoading ? (
          <Skeleton className="h-64 w-64" />
        ) : qrData?.qr_code ? (
          <div className="relative">
            <img src={qrData.qr_code} alt="QR Code" className="h-64 w-64 rounded-lg" />
            {isFetching && (
              <div className="absolute inset-0 bg-background/50 flex items-center justify-center rounded-lg">
                <Loader2 className="h-8 w-8 animate-spin" />
              </div>
            )}
          </div>
        ) : (
          <div className="h-64 w-64 flex items-center justify-center bg-muted rounded-lg">
            <p className="text-sm text-muted-foreground">QR code unavailable</p>
          </div>
        )}
        <div className="mt-4 flex items-center gap-4">
          {timeLeft !== null && <p className="text-sm text-muted-foreground">Expires in {timeLeft}s</p>}
          <Button variant="outline" size="sm" onClick={() => refetch()} disabled={isFetching}>
            <RefreshCw className={`mr-2 h-4 w-4 ${isFetching ? 'animate-spin' : ''}`} />Refresh
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
