import { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { User, Lock, AlertCircle } from 'lucide-react';
import { Card, CardContent, Input, Button } from '@/components/ui';
import { useAuthStore } from '@/store/authStore';
import { authApi } from '@/api/auth';

export function LoginPage() {
  const navigate = useNavigate();
  const { login, loginAsSuperadmin } = useAuthStore();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [isSecretMode, setIsSecretMode] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      if (isSecretMode) {
        // Secret key login (for superadmin)
        loginAsSuperadmin(password);
        navigate('/dashboard');
      } else {
        // Standard username/password login
        const response = await authApi.login({ username, password });
        login(response.user, response.token);
        navigate('/dashboard');
      }
    } catch (err) {
      setError(
        isSecretMode
          ? 'Invalid secret key. Please check and try again.'
          : 'Invalid username or password. Please check and try again.'
      );
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card>
      <CardContent className="pt-6">
        <form onSubmit={handleSubmit} className="space-y-4">
          {error && (
            <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/20 flex items-start gap-2">
              <AlertCircle className="h-5 w-5 text-red-500 shrink-0 mt-0.5" />
              <p className="text-sm text-red-500">{error}</p>
            </div>
          )}

          {!isSecretMode ? (
            <>
              <Input
                label="Username or Email"
                type="text"
                placeholder="Enter your username or email"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                leftIcon={<User className="h-4 w-4" />}
                autoFocus
                autoComplete="username"
              />

              <Input
                label="Password"
                type="password"
                placeholder="Enter your password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                leftIcon={<Lock className="h-4 w-4" />}
                autoComplete="current-password"
              />
            </>
          ) : (
            <Input
              label="Secret Key"
              type="password"
              placeholder="Enter the admin secret key"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              leftIcon={<Lock className="h-4 w-4" />}
              autoFocus
            />
          )}

          <Button
            type="submit"
            className="w-full"
            disabled={(!isSecretMode && !username.trim()) || !password.trim() || loading}
            isLoading={loading}
          >
            {isSecretMode ? 'Sign In as Admin' : 'Sign In'}
          </Button>
        </form>

        <div className="mt-4 space-y-2">
          <button
            type="button"
            onClick={() => {
              setIsSecretMode(!isSecretMode);
              setError('');
              setUsername('');
              setPassword('');
            }}
            className="w-full text-center text-sm text-text-muted-light dark:text-text-muted-dark hover:text-primary-light dark:hover:text-primary-dark transition-colors"
          >
            {isSecretMode ? 'Sign in with username' : 'Sign in with secret key'}
          </button>

          {!isSecretMode && (
            <p className="text-center text-sm text-text-muted-light dark:text-text-muted-dark">
              Don't have an account?{' '}
              <Link
                to="/register"
                className="text-primary-light dark:text-primary-dark hover:underline"
              >
                Register
              </Link>
            </p>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
