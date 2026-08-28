import { Navigate } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';
import type { JSX } from 'react';

export default function ProtectedRoute({ children }: { children: JSX.Element }) {
  const { user, loading } = useAuth();
  if (loading) return <div className="p-8 text-center text-zinc-500">Loading…</div>;
  if (!user) return <Navigate to="/login" replace />;
  return children;
}
