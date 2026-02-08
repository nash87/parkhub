import { useState, useEffect } from 'react';
import { Navigate } from 'react-router-dom';
import { SpinnerGap } from '@phosphor-icons/react';
import { useAuth } from '../context/AuthContext';
import { useSetupStatus } from '../components/SetupGuard';
import { OnboardingWizard } from '../components/OnboardingWizard';
import { useTranslation } from 'react-i18next';

export function SetupPage() {
  const { t } = useTranslation();
  const { isAuthenticated, isLoading: authLoading, login } = useAuth();
  const { setupComplete, recheckSetup } = useSetupStatus();
  const [autoLoginAttempted, setAutoLoginAttempted] = useState(false);
  const [autoLoginFailed, setAutoLoginFailed] = useState(false);

  // If setup is already complete, redirect to home
  if (setupComplete) {
    return <Navigate to="/" replace />;
  }

  // Auto-login as admin/admin if not authenticated
  useEffect(() => {
    if (!authLoading && !isAuthenticated && !autoLoginAttempted) {
      setAutoLoginAttempted(true);
      login('admin', 'admin').then(success => {
        if (!success) {
          setAutoLoginFailed(true);
        }
      });
    }
  }, [authLoading, isAuthenticated, autoLoginAttempted, login]);

  // Show loading while auth is loading or auto-login in progress
  if (authLoading || (!isAuthenticated && !autoLoginFailed)) {
    return (
      <div className="min-h-screen flex flex-col items-center justify-center bg-gray-50 dark:bg-gray-950 gap-3">
        <SpinnerGap weight="bold" className="w-8 h-8 text-primary-600 animate-spin" />
        <p className="text-sm text-gray-500 dark:text-gray-400">{t('onboarding.preparingSetup')}</p>
      </div>
    );
  }

  // If auto-login failed (shouldn't happen on fresh install), redirect to login
  if (autoLoginFailed && !isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  // Show the onboarding wizard as a full-page experience
  return (
    <OnboardingWizard
      onComplete={async () => {
        await recheckSetup();
      }}
    />
  );
}
