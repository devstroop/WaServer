import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Key } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/Card';
import { useAuthenticate } from '../hooks/useAuth';
import { useAuthStore } from '@/stores';

const apiKeySchema = z.object({
  apiKey: z.string().min(1, 'API key is required').min(8, 'API key must be at least 8 characters'),
});

type ApiKeyForm = z.infer<typeof apiKeySchema>;

export function LoginForm() {
  const authenticate = useAuthenticate();
  const isValidating = useAuthStore((state) => state.isValidating);
  const { register, handleSubmit, formState: { errors } } = useForm<ApiKeyForm>({
    resolver: zodResolver(apiKeySchema),
  });

  const onSubmit = (data: ApiKeyForm) => {
    authenticate.mutate(data.apiKey);
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Key className="h-5 w-5" />
          Connect to WAS
        </CardTitle>
        <CardDescription>
          Enter your API secret key to authenticate with the WhatsApp Server.
          The key is configured in your server's <code className="text-xs bg-muted px-1 rounded">app.toml</code> file.
        </CardDescription>
      </CardHeader>
      <form onSubmit={handleSubmit(onSubmit)}>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="apiKey">API Secret Key</Label>
            <Input
              id="apiKey"
              type="password"
              placeholder="Enter your secret key..."
              {...register('apiKey')}
              error={!!errors.apiKey}
              autoComplete="off"
            />
            {errors.apiKey && <p className="text-sm text-destructive">{errors.apiKey.message}</p>}
          </div>
          <p className="text-xs text-muted-foreground">
            The API key is set via <code className="bg-muted px-1 rounded">auth.secret_key</code> in your config
            or the <code className="bg-muted px-1 rounded">WAS__AUTH__SECRET_KEY</code> environment variable.
          </p>
        </CardContent>
        <CardFooter>
          <Button type="submit" className="w-full" loading={authenticate.isPending || isValidating}>
            {isValidating ? 'Validating...' : 'Connect'}
          </Button>
        </CardFooter>
      </form>
    </Card>
  );
}
