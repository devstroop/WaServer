import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import { AuthProvider } from './hooks/useAuth';
import Layout from './components/Layout';
import ProtectedRoute from './components/ProtectedRoute';
import Dashboard from './pages/Dashboard';
import Home from './pages/Home';
import Instances from './pages/Instances';
import Login from './pages/Login';
import Register from './pages/Register';

function NotImplemented() {
  return <div>Not implemented</div>;
}

export default function App() {
  return (
    <AuthProvider>
      <BrowserRouter>
        <Routes>
          {/* Public */}
          <Route path="/" element={<Home />} />

          {/* Auth */}
          <Route path="/auth/login" element={<Login />} />
          <Route path="/auth/register" element={<Register />} />

          {/* Protected zones: user + admin (guards placeholder, shells deferred) */}
          <Route
            element={
              <ProtectedRoute>
                <Layout />
              </ProtectedRoute>
            }
          >
            {/* User */}
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/dashboard/instances" element={<Instances />} />
            <Route path="/dashboard/instances/:id" element={<NotImplemented />} />

            {/* Admin */}
            <Route path="/admin" element={<NotImplemented />} />
            <Route path="/admin/dashboard" element={<NotImplemented />} />
            <Route path="/admin/metrics" element={<NotImplemented />} />
            <Route path="/admin/instances" element={<NotImplemented />} />
            <Route path="/admin/instances/:id" element={<NotImplemented />} />
            <Route path="/admin/users" element={<NotImplemented />} />
            <Route path="/admin/users/:id" element={<NotImplemented />} />
            <Route path="/settings" element={<NotImplemented />} />
          </Route>

          {/* Fallback */}
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </BrowserRouter>
    </AuthProvider>
  );
}
