import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Check } from 'lucide-react';
import { Header } from '@/components/layout';
import { Card, CardContent, CardHeader, CardTitle, Input, Button, Select } from '@/components/ui';
import { usersApi } from '@/api/users';

export function NewUserPage() {
  const navigate = useNavigate();
  const [username, setUsername] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [role, setRole] = useState<'admin' | 'user'>('user');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    // Validate passwords match
    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    // Validate password length
    if (password.length < 8) {
      setError('Password must be at least 8 characters');
      return;
    }

    setLoading(true);

    try {
      await usersApi.createUser({
        username,
        email: email || undefined,
        password,
        role,
      });
      setSuccess(true);
    } catch (err: unknown) {
      if (err && typeof err === 'object' && 'message' in err) {
        setError((err as { message: string }).message);
      } else {
        setError('Failed to create user');
      }
    } finally {
      setLoading(false);
    }
  };

  if (success) {
    return (
      <>
        <Header title="User Created" />
        <div className="p-6 max-w-lg">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Check className="h-5 w-5 text-green-500" />
                User Created Successfully
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="p-4 rounded-lg bg-green-500/10 border border-green-500/20">
                <p className="text-sm text-green-600 dark:text-green-400">
                  User <strong>{username}</strong> has been created with the <strong>{role}</strong> role.
                </p>
              </div>

              <p className="text-sm text-text-muted-light dark:text-text-muted-dark">
                The user can now log in with their username and password.
              </p>

              <Button onClick={() => navigate('/users')} className="w-full">
                Go to Users
              </Button>
            </CardContent>
          </Card>
        </div>
      </>
    );
  }

  return (
    <>
      <Header
        title="New User"
        description="Create a new user account"
      />

      <div className="p-6 max-w-lg">
        <Card>
          <CardHeader>
            <CardTitle>User Details</CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              {error && (
                <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/20">
                  <p className="text-sm text-red-500">{error}</p>
                </div>
              )}

              <Input
                label="Username"
                placeholder="john_doe"
                value={username}
                onChange={(e) => setUsername(e.target.value.toLowerCase().replace(/[^a-z0-9_]/g, ''))}
                error={username.length > 0 && username.length < 3 ? 'Minimum 3 characters' : undefined}
                required
              />

              <Input
                label="Email (optional)"
                type="email"
                placeholder="john@example.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
              />

              <Input
                label="Password"
                type="password"
                placeholder="Minimum 8 characters"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                error={password.length > 0 && password.length < 8 ? 'Minimum 8 characters' : undefined}
                required
              />

              <Input
                label="Confirm Password"
                type="password"
                placeholder="Confirm password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                error={confirmPassword.length > 0 && confirmPassword !== password ? 'Passwords do not match' : undefined}
                required
              />

              <Select
                label="Role"
                value={role}
                onChange={(e) => setRole(e.target.value as 'admin' | 'user')}
                options={[
                  { value: 'user', label: 'User - Access only assigned instances' },
                  { value: 'admin', label: 'Admin - Full access to all instances' },
                ]}
              />

              <div className="flex gap-3 pt-2">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => navigate('/users')}
                >
                  Cancel
                </Button>
                <Button
                  type="submit"
                  disabled={username.length < 3 || password.length < 8 || password !== confirmPassword || loading}
                  isLoading={loading}
                >
                  Create User
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      </div>
    </>
  );
}
