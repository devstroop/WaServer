import { useEffect, useState, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { RefreshCw, CheckCircle, AlertCircle } from 'lucide-react';
import { Header } from '@/components/layout';
import { Card, CardContent, CardHeader, CardTitle, Button, Skeleton } from '@/components/ui';
import { instancesApi } from '@/api/instances';

type LinkingStatus = 'loading' | 'qr' | 'connected' | 'error';

export function LinkInstancePage() {
  const { instanceId } = useParams<{ instanceId: string }>();
  const navigate = useNavigate();
  const [status, setStatus] = useState<LinkingStatus>('loading');
  const [qrCode, setQrCode] = useState<string | null>(null);
  const [error, setError] = useState('');
  const [polling, setPolling] = useState(false);

  const startLinking = useCallback(async () => {
    if (!instanceId) return;
    
    setStatus('loading');
    setError('');
    
    try {
      const session = await instancesApi.linkInstance(instanceId);
      if (session.qr_code) {
        setQrCode(session.qr_code);
        setStatus('qr');
        setPolling(true);
      } else if (session.status === 'connected') {
        setStatus('connected');
      }
    } catch (err: any) {
      setError(err.message || 'Failed to start linking');
      setStatus('error');
    }
  }, [instanceId]);

  // Poll for status updates
  useEffect(() => {
    if (!polling || !instanceId) return;

    const interval = setInterval(async () => {
      try {
        const instance = await instancesApi.getInstance(instanceId);
        if (instance.status === 'connected') {
          setStatus('connected');
          setPolling(false);
        }
      } catch (err) {
        // Ignore polling errors
      }
    }, 3000);

    return () => clearInterval(interval);
  }, [polling, instanceId]);

  useEffect(() => {
    startLinking();
  }, [startLinking]);

  return (
    <>
      <Header 
        title="Link Instance" 
        description={`Connect ${instanceId} to WhatsApp`}
      />

      <div className="p-6 max-w-lg mx-auto">
        <Card>
          <CardHeader className="text-center">
            <CardTitle>
              {status === 'loading' && 'Generating QR Code...'}
              {status === 'qr' && 'Scan QR Code'}
              {status === 'connected' && 'Connected!'}
              {status === 'error' && 'Connection Failed'}
            </CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col items-center">
            {status === 'loading' && (
              <div className="space-y-4 w-full">
                <Skeleton className="h-64 w-64 mx-auto" />
                <p className="text-center text-text-muted-light dark:text-text-muted-dark">
                  Preparing WhatsApp connection...
                </p>
              </div>
            )}

            {status === 'qr' && qrCode && (
              <div className="space-y-4">
                <div className="p-4 bg-white rounded-xl">
                  <img 
                    src={`data:image/png;base64,${qrCode}`} 
                    alt="WhatsApp QR Code"
                    className="h-64 w-64"
                  />
                </div>
                
                <div className="space-y-2 text-center">
                  <p className="text-text-light dark:text-text-dark font-medium">
                    Open WhatsApp on your phone
                  </p>
                  <ol className="text-sm text-text-muted-light dark:text-text-muted-dark space-y-1">
                    <li>1. Tap <strong>Menu</strong> or <strong>Settings</strong></li>
                    <li>2. Tap <strong>Linked Devices</strong></li>
                    <li>3. Tap <strong>Link a Device</strong></li>
                    <li>4. Point your phone at this screen</li>
                  </ol>
                </div>

                <Button
                  variant="outline"
                  onClick={startLinking}
                  className="w-full"
                >
                  <RefreshCw className="h-4 w-4 mr-2" />
                  Refresh QR Code
                </Button>
              </div>
            )}

            {status === 'connected' && (
              <div className="space-y-4 text-center py-8">
                <div className="h-20 w-20 rounded-full bg-green-500/10 flex items-center justify-center mx-auto">
                  <CheckCircle className="h-10 w-10 text-green-500" />
                </div>
                <div>
                  <p className="text-lg font-medium text-text-light dark:text-text-dark">
                    Successfully Connected
                  </p>
                  <p className="text-text-muted-light dark:text-text-muted-dark mt-1">
                    Your WhatsApp instance is now ready to use.
                  </p>
                </div>
                <Button onClick={() => navigate(`/instances/${instanceId}`)}>
                  Go to Instance
                </Button>
              </div>
            )}

            {status === 'error' && (
              <div className="space-y-4 text-center py-8">
                <div className="h-20 w-20 rounded-full bg-red-500/10 flex items-center justify-center mx-auto">
                  <AlertCircle className="h-10 w-10 text-red-500" />
                </div>
                <div>
                  <p className="text-lg font-medium text-text-light dark:text-text-dark">
                    Connection Failed
                  </p>
                  <p className="text-text-muted-light dark:text-text-muted-dark mt-1">
                    {error || 'Unable to generate QR code. Please try again.'}
                  </p>
                </div>
                <div className="flex gap-3 justify-center">
                  <Button variant="outline" onClick={() => navigate('/instances')}>
                    Back to Instances
                  </Button>
                  <Button onClick={startLinking}>
                    Try Again
                  </Button>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </>
  );
}
