import { AuthLayout } from '@/layouts';
import { LoginForm } from '@/features/auth';

export function LoginPage() {
  return (
    <AuthLayout>
      <LoginForm />
    </AuthLayout>
  );
}
