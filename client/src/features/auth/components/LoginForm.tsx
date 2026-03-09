import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Key, Eye, EyeOff } from 'lucide-react';
import { useState } from 'react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import { useAuthenticate } from '../hooks/useAuth';
import { useAuthStore } from '@/stores';

const apiKeySchema = z.object({
  apiKey: z.string().min(1, 'API key is required').min(8, 'API key must be at least 8 characters'),
});

type ApiKeyForm = z.infer<typeof apiKeySchema>;

export function LoginForm() {
  const [showKey, setShowKey] = useState(false);
  const authenticate = useAuthenticate();
  const isValidating = useAuthStore((state) => state.isValidating);
  const { register, handleSubmit, formState: { errors } } = useForm<ApiKeyForm>({
    resolver: zodResolver(apiKeySchema),
  });

  const onSubmit = (data: ApiKeyForm) => {
    authenticate.mutate(data.apiKey);
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
      <div className="space-y-2">
        <Label htmlFor="apiKey" className="text-sm font-medium">
          API Secret Key
        </Label>
        <div className="relative">
          <div className="absolute left-0 top-0 h-full w-12 flex items-center justify-center border-r bg-muted/50 rounded-l-md">
            <Key className="h-4 w-4 text-muted-foreground" />
          </div>
          <Input
            id="apiKey"
            type={showKey ? 'text' : 'password'}
            placeholder="sk_live_xxxxxxxxxxxxxxxx"
            className="pl-14 pr-12 h-12 font-mono text-sm"
            {...register('apiKey')}
            error={!!errors.apiKey}
            autoComplete="off"
          />
          <button
            type="button"
            onClick={() => setShowKey(!showKey)}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
          >
            {showKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
          </button>
        </div>
        {errors.apiKey && (
          <p className="text-sm text-destructive flex items-center gap-1">
            <span className="inline-block w-1 h-1 rounded-full bg-destructive" />
            {errors.apiKey.message}
          </p>
        )}
      </div>

      <div className="rounded-lg border bg-muted/30 p-4">
        <p className="text-xs text-muted-foreground leading-relaxed">
          <span className="font-semibold text-foreground block mb-1">Where to find your API key?</span>
          Check your <code className="bg-background px-1.5 py-0.5 rounded border text-[11px] font-mono">app.toml</code>{' '}
          config file or set the <code className="bg-background px-1.5 py-0.5 rounded border text-[11px] font-mono">WAS__AUTH__SECRET_KEY</code>{' '}
          environment variable.
        </p>
      </div>

      <Button 
        type="submit" 
        className="w-full h-12 text-base font-semibold shadow-lg shadow-primary/25 hover:shadow-primary/40 transition-shadow" 
        loading={authenticate.isPending || isValidating}
      >
        {isValidating ? 'Validating...' : 'Sign in to Dashboard'}
      </Button>
    </form>
  );
}
