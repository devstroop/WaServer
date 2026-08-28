import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider } from './hooks/useAuth';
import Layout from './components/Layout';
import ProtectedRoute from './components/ProtectedRoute';
import Home from './pages/Home';
import Login from './pages/Login';
import Register from './pages/Register';
import Dashboard from './pages/Dashboard';
import Instances from './pages/Instances';
import InstanceDetail from './pages/InstanceDetail';

export default function App() {
  return (
    <AuthProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/login" element={<Login />} />
          <Route path="/register" element={<Register />} />
          <Route
            element={
              <ProtectedRoute>
                <Layout />
              </ProtectedRoute>
            }
          >
            <Route path="app" element={<Dashboard />} />
            <Route path="app/instances" element={<Instances />} />
            <Route path="app/instances/:id" element={<InstanceDetail />} />
            <Route path="dashboard" element={<Dashboard />} />
            <Route path="dashboard/instances" element={<Instances />} />
            <Route path="dashboard/instances/:id" element={<InstanceDetail />} />
            <Route path="admin" element={<Dashboard />} />
            <Route path="admin/instances" element={<Instances />} />
            <Route path="admin/instances/:id" element={<InstanceDetail />} />
          </Route>
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </BrowserRouter>
    </AuthProvider>
  );
}
