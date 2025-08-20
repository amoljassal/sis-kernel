import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Toaster } from 'react-hot-toast';

// Components
import Layout from './components/Layout';
import Home from './pages/Home';
import Dashboard from './pages/Dashboard';
import Settings from './pages/Settings';
import Pricing from './pages/Pricing';

// Contexts
import { AuthProvider } from './contexts/AuthContext';
import { BillingProvider } from './contexts/BillingContext';

// Analytics
import { initAnalytics } from './lib/analytics';

// Create a client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      cacheTime: 10 * 60 * 1000, // 10 minutes
    },
  },
});

// Initialize analytics
initAnalytics();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <BillingProvider>
          <Router>
            <div className="min-h-screen bg-gray-50">
              <Layout>
                <Routes>
                  <Route path="/" element={<Home />} />
                  <Route path="/dashboard" element={<Dashboard />} />
                  <Route path="/settings" element={<Settings />} />
                  <Route path="/pricing" element={<Pricing />} />
                </Routes>
              </Layout>
              <Toaster 
                position="top-right"
                toastOptions={{
                  duration: 4000,
                  style: {
                    background: '#363636',
                    color: '#fff',
                  },
                }}
              />
            </div>
          </Router>
        </BillingProvider>
      </AuthProvider>
    </QueryClientProvider>
  );
}

export default App;