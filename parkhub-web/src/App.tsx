import { useEffect, lazy, Suspense } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { Toaster } from 'react-hot-toast';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AuthProvider, useAuth } from './context/AuthContext';
import { BrandingProvider } from './context/BrandingContext';
import { useTheme, applyTheme } from './stores/theme';
import { useAccessibility, applyAccessibility } from './stores/accessibility';
import { useTranslation } from 'react-i18next';
import { Layout } from './components/Layout';
import { LoginPage } from './pages/Login';
import { RegisterPage } from './pages/Register';
import { DashboardPage } from './pages/Dashboard';
import { BookPage } from './pages/Book';
import { BookingsPage } from './pages/Bookings';
import { VehiclesPage } from './pages/Vehicles';
import { ConsentBanner } from './components/ConsentBanner';
import { OnboardingWizard } from './components/OnboardingWizard';
import { SpinnerGap } from '@phosphor-icons/react';

const AdminPage = lazy(() => import('./pages/Admin').then(m => ({ default: m.AdminPage })));
const HomeofficePage = lazy(() => import('./pages/Homeoffice').then(m => ({ default: m.HomeofficePage })));
const ProfilePage = lazy(() => import('./pages/Profile').then(m => ({ default: m.ProfilePage })));
const PrivacyPage = lazy(() => import('./pages/Privacy').then(m => ({ default: m.PrivacyPage })));
const TermsPage = lazy(() => import('./pages/Terms').then(m => ({ default: m.TermsPage })));
const LegalPage = lazy(() => import('./pages/Legal').then(m => ({ default: m.LegalPage })));
const AboutPage = lazy(() => import('./pages/About').then(m => ({ default: m.AboutPage })));
const HelpPage = lazy(() => import('./pages/Help').then(m => ({ default: m.HelpPage })));

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60,
      retry: 1,
    },
  },
});

function LoadingScreen() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-950">
      <SpinnerGap weight="bold" className="w-8 h-8 text-primary-600 animate-spin" />
    </div>
  );
}

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, isLoading } = useAuth();
  if (isLoading) return <LoadingScreen />;
  if (!isAuthenticated) return <Navigate to="/login" replace />;
  return <Layout>{children}</Layout>;
}

function AdminRoute({ children }: { children: React.ReactNode }) {
  const { user, isAuthenticated, isLoading } = useAuth();
  if (isLoading) return <LoadingScreen />;
  if (!isAuthenticated) return <Navigate to="/login" replace />;
  if (user?.role !== 'admin' && user?.role !== 'superadmin') return <Navigate to="/" replace />;
  return <Layout>{children}</Layout>;
}

function PublicPageWithLayout({ children }: { children: React.ReactNode }) {
  return <>{children}</>;
}

function ThemeInitializer({ children }: { children: React.ReactNode }) {
  const theme = useTheme();
  const accessibility = useAccessibility();
  const { i18n } = useTranslation();

  useEffect(() => { applyTheme(theme.isDark); }, [theme.isDark]);
  useEffect(() => { applyAccessibility(accessibility); }, [accessibility.colorMode, accessibility.fontScale, accessibility.reducedMotion, accessibility.highContrast]);
  useEffect(() => { document.documentElement.lang = i18n.language?.startsWith('en') ? 'en' : 'de'; }, [i18n.language]);

  return <>{children}</>;
}

function OnboardingGuard() {
  const { user } = useAuth();
  if (user?.role !== 'admin' && user?.role !== 'superadmin') return null;
  return <OnboardingWizard onComplete={() => {}} />;
}

function AppRoutes() {
  return (
    <Routes>
      {/* Public */}
      <Route path="/login" element={<LoginPage />} />
      <Route path="/register" element={<RegisterPage />} />
      <Route path="/privacy" element={<PublicPageWithLayout><Suspense fallback={<LoadingScreen />}><PrivacyPage /></Suspense></PublicPageWithLayout>} />
      <Route path="/terms" element={<PublicPageWithLayout><Suspense fallback={<LoadingScreen />}><TermsPage /></Suspense></PublicPageWithLayout>} />
      <Route path="/legal" element={<PublicPageWithLayout><Suspense fallback={<LoadingScreen />}><LegalPage /></Suspense></PublicPageWithLayout>} />
      <Route path="/about" element={<PublicPageWithLayout><Suspense fallback={<LoadingScreen />}><AboutPage /></Suspense></PublicPageWithLayout>} />
      <Route path="/help" element={<ProtectedRoute><Suspense fallback={<LoadingScreen />}><HelpPage /></Suspense></ProtectedRoute>} />

      {/* Protected */}
      <Route path="/" element={<ProtectedRoute><DashboardPage /></ProtectedRoute>} />
      <Route path="/book" element={<ProtectedRoute><BookPage /></ProtectedRoute>} />
      <Route path="/bookings" element={<ProtectedRoute><BookingsPage /></ProtectedRoute>} />
      <Route path="/vehicles" element={<ProtectedRoute><VehiclesPage /></ProtectedRoute>} />
      <Route path="/homeoffice" element={<ProtectedRoute><Suspense fallback={<LoadingScreen />}><HomeofficePage /></Suspense></ProtectedRoute>} />
      <Route path="/profile" element={<ProtectedRoute><Suspense fallback={<LoadingScreen />}><ProfilePage /></Suspense></ProtectedRoute>} />

      {/* Admin */}
      <Route path="/admin/*" element={<AdminRoute><Suspense fallback={<LoadingScreen />}><AdminPage /></Suspense></AdminRoute>} />

      {/* Catch all */}
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <ThemeInitializer>
          <BrandingProvider>
            <AuthProvider>
            <AppRoutes />
            <OnboardingGuard />
            <ConsentBanner />
            <Toaster
              position="top-right"
              toastOptions={{
                duration: 4000,
                style: {
                  background: 'var(--toast-bg, #fff)',
                  color: 'var(--toast-color, #1f2937)',
                  borderRadius: '12px',
                  boxShadow: '0 10px 40px -10px rgba(0, 0, 0, 0.2)',
                },
                success: { iconTheme: { primary: '#22c55e', secondary: '#fff' } },
                error: { iconTheme: { primary: '#ef4444', secondary: '#fff' } },
              }}
            />
          </AuthProvider>
          </BrandingProvider>
        </ThemeInitializer>
      </BrowserRouter>
    </QueryClientProvider>
  );
}

export default App;
