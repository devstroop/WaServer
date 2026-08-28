import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider } from './hooks/useAuth';
import Layout from './components/Layout';
import ProtectedRoute from './components/ProtectedRoute';
import Home from './pages/Home';
import Login from './pages/Login';
import Register from './pages/Register';
import Dashboard from './pages/Dashboard';
import { InstanceList } from './pages/Instances';

function NotImplemented() {
  return <div className="p-8 text-center text-zinc-500">Not implemented</div>;
}

export default function App() {
  return (
    <AuthProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/login" element={<Login />} />
          <Route path="/register" element={<Register />} />
          <Route path="/auth/login" element={<Login />} />
          <Route path="/auth/register" element={<Register />} />
          <Route
            element={
              <ProtectedRoute>
                <Layout />
              </ProtectedRoute>
            }
          >
            <Route path="app" element={<Dashboard />} />
            <Route path="app/instances" element={<InstanceList />} />
            <Route path="dashboard" element={<Dashboard />} />
            <Route path="dashboard/instances" element={<InstanceList />} />
            <Route path="dashboard/instances/:id" element={<NotImplemented />} />
            <Route path="admin" element={<NotImplemented />} />
            <Route path="admin/dashboard" element={<NotImplemented />} />
            <Route path="admin/metrics" element={<NotImplemented />} />
            <Route path="admin/instances" element={<InstanceList admin />} />
            <Route path="admin/instances/:id" element={<NotImplemented />} />
            <Route path="admin/users" element={<NotImplemented />} />
            <Route path="admin/users/:id" element={<NotImplemented />} />
            <Route path="settings" element={<NotImplemented />} />
          </Route>
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </BrowserRouter>
    </AuthProvider>
  );
}
