import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';
import { Lock, Buildings, Car, Users, CheckCircle, ArrowRight, ArrowLeft, Database, Sparkle } from '@phosphor-icons/react';
import { useTranslation } from 'react-i18next';

// Auth context available via provider
import toast from 'react-hot-toast';

function getAuthHeaders(): Record<string, string> {
  const token = localStorage.getItem('parkhub_token');
  return token ? { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' } : { 'Content-Type': 'application/json' };
}

interface OnboardingWizardProps {
  onComplete: () => void;
}

export function OnboardingWizard({ onComplete }: OnboardingWizardProps) {
  const { t } = useTranslation();
  
  const navigate = useNavigate();
  const [step, setStep] = useState(0);
  const [visible, setVisible] = useState(false);
  const [loading, setLoading] = useState(true);

  // Form state
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [passwordError, setPasswordError] = useState('');
  const [passwordChanged, setPasswordChanged] = useState(false);
  const [companyName, setCompanyName] = useState('');
  const [loadDummyData, setLoadDummyData] = useState<boolean | null>(null);
  const [selfRegistration, setSelfRegistration] = useState(false);

  // Setup status from API
  

  useEffect(() => {
    checkSetupStatus();
  }, []);

  async function checkSetupStatus() {
    try {
      const res = await fetch('/api/v1/setup/status');
      const data = await res.json();
      if (data.success && data.data) {
        // setup status loaded
        // Show wizard if setup not complete
        if (!data.data.setup_complete) {
          setVisible(true);
          // If password needs changing, force step 0
          if (data.data.needs_password_change) {
            setStep(0);
          }
        }
      }
    } catch {
      // If API fails, check localStorage fallback
      if (!localStorage.getItem('parkhub_onboarding_done')) {
        setVisible(true);
      }
    } finally {
      setLoading(false);
    }
  }

  async function handlePasswordChange() {
    setPasswordError('');
    if (newPassword.length < 8) {
      setPasswordError(t('register.passwordTooShort'));
      return false;
    }
    if (newPassword !== confirmPassword) {
      setPasswordError(t('register.passwordMismatch'));
      return false;
    }

    try {
      // Always fresh-login first to ensure valid token
      const loginRes = await fetch('/api/v1/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username: 'admin', password: currentPassword }),
      });
      const loginData = await loginRes.json();
      if (!loginData.success) {
        setPasswordError('Aktuelles Passwort ist falsch');
        return false;
      }
      const freshToken = loginData.data.tokens.access_token;
      localStorage.setItem('parkhub_token', freshToken);
      if (loginData.data.tokens.refresh_token)
        localStorage.setItem('parkhub_refresh_token', loginData.data.tokens.refresh_token);

      // Now change password with guaranteed fresh token
      const res = await fetch('/api/v1/users/me/password', {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${freshToken}` },
        body: JSON.stringify({ current_password: currentPassword, new_password: newPassword }),
      });
      const data = await res.json();
      if (data.success) {
        setPasswordChanged(true);
        toast.success('Passwort erfolgreich geändert');
        return true;
      } else {
        setPasswordError(data.error?.message || 'Fehler beim Ändern des Passworts');
        return false;
      }
    } catch {
      setPasswordError('Netzwerkfehler');
      return false;
    }
  }

  async function handleCompanySave() {
    if (!companyName.trim()) return true; // Skip if empty
    try {
      await fetch('/api/v1/admin/branding', {
        method: 'PUT',
        headers: getAuthHeaders(),
        body: JSON.stringify({
          company_name: companyName,
          primary_color: '#3B82F6',
          secondary_color: '#1D4ED8',
          login_background_color: '#2563EB',
        }),
      });
      return true;
    } catch {
      return true; // Don't block on branding errors
    }
  }

  async function handleDummyData() {
    if (!loadDummyData) return true;
    // The server already has create_sample_parking_lot function
    // We need to trigger it via API - create a lot manually
    try {
      
      const lotId = crypto.randomUUID();
      const slots = Array.from({ length: 10 }, (_, i) => {
        const slotId = crypto.randomUUID();
        return {
          id: slotId,
          number: `P${i + 1}`,
          status: 'available',
        };
      });
      const lot = {
        id: lotId,
        name: 'Beispiel-Parkplatz',
        address: 'Hauptstraße 1',
        total_slots: 10,
        available_slots: 10,
        layout: {
          rows: [
            { id: crypto.randomUUID(), label: 'Reihe A', side: 'top', slots: slots.slice(0, 5) },
            { id: crypto.randomUUID(), label: 'Reihe B', side: 'bottom', slots: slots.slice(5, 10) },
          ],
          road_label: 'Fahrweg',
        },
        status: 'open',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      await fetch('/api/v1/lots', {
        method: 'POST',
        headers: getAuthHeaders(),
        body: JSON.stringify(lot),
      });
      toast.success('Beispieldaten geladen');
      return true;
    } catch {
      return true;
    }
  }

  async function completeSetup() {
    try {
      await fetch('/api/v1/setup/complete', {
        method: 'POST',
        headers: getAuthHeaders(),
      });
    } catch { /* ignore */ }
    localStorage.setItem('parkhub_onboarding_done', 'true');
    setVisible(false);
    onComplete();
    navigate('/');
  }

  async function handleNext() {
    if (step === 0) {
      // Password change step - required
      if (!passwordChanged) {
        const ok = await handlePasswordChange();
        if (!ok) return;
      }
      setStep(1);
    } else if (step === 1) {
      // Company setup
      await handleCompanySave();
      setStep(2);
    } else if (step === 2) {
      // Dummy data question
      if (loadDummyData) {
        await handleDummyData();
        setStep(4); // Skip lot creation, go to users
      } else if (loadDummyData === false) {
        setStep(3); // Go to manual lot creation
      }
    } else if (step === 3) {
      // Lot creation - user can do it in admin later
      setStep(4);
    } else if (step === 4) {
      // User management
      setStep(5);
    } else if (step === 5) {
      // Done
      await completeSetup();
    }
  }

  if (loading || !visible) return null;

  const steps = [
    { icon: Lock, key: 'password' },
    { icon: Buildings, key: 'company' },
    { icon: Database, key: 'dummyData' },
    { icon: Car, key: 'lot' },
    { icon: Users, key: 'users' },
    { icon: CheckCircle, key: 'done' },
  ];

  // const Icon = steps[step].icon;
  const isLast = step === steps.length - 1;
  const canGoNext = step === 0 ? (passwordChanged || (currentPassword && newPassword && confirmPassword)) :
                    step === 2 ? loadDummyData !== null :
                    true;

  return (
    <div className="fixed inset-0 z-[70] flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
      <motion.div
        initial={{ scale: 0.9, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        className="card w-full max-w-lg p-6 sm:p-8 shadow-2xl max-h-[90vh] overflow-y-auto"
      >
        <div className="flex items-center justify-between mb-4 sm:mb-6">
          <h2 className="text-lg sm:text-xl font-bold text-gray-900 dark:text-white">
            {step === 0 ? 'Willkommen bei ParkHub!' : t('onboarding.title')}
          </h2>
          <span className="text-xs text-gray-400">{step + 1}/{steps.length}</span>
        </div>

        {/* Progress */}
        <div className="flex gap-1 mb-6 sm:mb-8">
          {steps.map((_, i) => (
            <div key={i} className={`flex-1 h-1.5 rounded-full transition-colors ${i <= step ? 'bg-primary-500' : 'bg-gray-200 dark:bg-gray-700'}`} />
          ))}
        </div>

        <AnimatePresence mode="wait">
          <motion.div
            key={step}
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
          >
            {/* Step 0: Password Change */}
            {step === 0 && (
              <div className="space-y-4">
                <div className="text-center mb-4">
                  <div className="w-16 h-16 mx-auto rounded-2xl bg-red-100 dark:bg-red-900/30 flex items-center justify-center mb-3">
                    <Lock weight="fill" className="w-8 h-8 text-red-600 dark:text-red-400" />
                  </div>
                  <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                    Bitte ändern Sie Ihr Admin-Passwort
                  </h3>
                  <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                    Das Standard-Passwort muss aus Sicherheitsgründen geändert werden.
                  </p>
                  <p className="text-xs text-gray-400 dark:text-gray-500 mt-1 bg-gray-100 dark:bg-gray-800 rounded-lg px-3 py-2">
                    📋 Standard-Login: Benutzername <strong>admin</strong> / Passwort <strong>admin</strong>
                  </p>
                </div>
                {passwordChanged ? (
                  <div className="p-4 bg-emerald-50 dark:bg-emerald-900/20 rounded-xl text-center">
                    <CheckCircle weight="fill" className="w-8 h-8 text-emerald-500 mx-auto mb-2" />
                    <p className="font-medium text-emerald-700 dark:text-emerald-300">Passwort erfolgreich geändert!</p>
                  </div>
                ) : (
                  <>
                    <div>
                      <label className="label">Aktuelles Passwort</label>
                      <input type="password" className="input" value={currentPassword}
                        onChange={e => setCurrentPassword(e.target.value)} placeholder="Standard: admin" />
                    </div>
                    <div>
                      <label className="label">Neues Passwort</label>
                      <input type="password" className="input" value={newPassword}
                        onChange={e => setNewPassword(e.target.value)} placeholder="Mindestens 8 Zeichen" />
                    </div>
                    <div>
                      <label className="label">Passwort bestätigen</label>
                      <input type="password" className="input" value={confirmPassword}
                        onChange={e => setConfirmPassword(e.target.value)} placeholder="Passwort wiederholen" />
                    </div>
                    {passwordError && <p className="text-sm text-red-500">{passwordError}</p>}
                  </>
                )}
              </div>
            )}

            {/* Step 1: Company Setup */}
            {step === 1 && (
              <div className="space-y-4">
                <div className="text-center mb-4">
                  <div className="w-16 h-16 mx-auto rounded-2xl bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center mb-3">
                    <Buildings weight="fill" className="w-8 h-8 text-primary-600 dark:text-primary-400" />
                  </div>
                  <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                    {t('onboarding.steps.company.title')}
                  </h3>
                  <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                    {t('onboarding.steps.company.desc')}
                  </p>
                </div>
                <div>
                  <label className="label">Firmenname</label>
                  <input type="text" className="input" value={companyName}
                    onChange={e => setCompanyName(e.target.value)} placeholder="z.B. Meine Firma GmbH" />
                </div>
              </div>
            )}

            {/* Step 2: Dummy Data */}
            {step === 2 && (
              <div className="space-y-4">
                <div className="text-center mb-4">
                  <div className="w-16 h-16 mx-auto rounded-2xl bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center mb-3">
                    <Database weight="fill" className="w-8 h-8 text-amber-600 dark:text-amber-400" />
                  </div>
                  <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                    Beispieldaten laden?
                  </h3>
                  <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                    Möchten Sie einen Beispiel-Parkplatz mit 10 Stellplätzen erstellen?
                  </p>
                </div>
                <div className="grid grid-cols-2 gap-3">
                  <button
                    onClick={() => setLoadDummyData(true)}
                    className={`p-4 rounded-xl border-2 text-center transition-all ${loadDummyData === true ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20' : 'border-gray-200 dark:border-gray-700 hover:border-gray-300'}`}
                  >
                    <Sparkle weight="fill" className="w-6 h-6 mx-auto mb-2 text-primary-500" />
                    <p className="font-medium text-sm">Ja, Beispieldaten</p>
                    <p className="text-xs text-gray-400 mt-1">Schnellstart mit Demodaten</p>
                  </button>
                  <button
                    onClick={() => setLoadDummyData(false)}
                    className={`p-4 rounded-xl border-2 text-center transition-all ${loadDummyData === false ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20' : 'border-gray-200 dark:border-gray-700 hover:border-gray-300'}`}
                  >
                    <Buildings weight="regular" className="w-6 h-6 mx-auto mb-2 text-gray-400" />
                    <p className="font-medium text-sm">Nein, leer starten</p>
                    <p className="text-xs text-gray-400 mt-1">Selbst konfigurieren</p>
                  </button>
                </div>
              </div>
            )}

            {/* Step 3: Lot Creation hint */}
            {step === 3 && (
              <div className="space-y-4 text-center">
                <div className="w-16 h-16 mx-auto rounded-2xl bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center mb-3">
                  <Car weight="fill" className="w-8 h-8 text-primary-600 dark:text-primary-400" />
                </div>
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                  {t('onboarding.steps.lot.title')}
                </h3>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  Sie können Ihren ersten Parkplatz jetzt unter <strong>Admin → Parkplätze</strong> erstellen.
                  Nutzen Sie den visuellen Layout-Editor, um Reihen und Stellplätze anzulegen.
                </p>
                <p className="text-xs text-gray-400">
                  Dieser Schritt kann auch später erledigt werden.
                </p>
              </div>
            )}

            {/* Step 4: Users */}
            {step === 4 && (
              <div className="space-y-4">
                <div className="text-center mb-4">
                  <div className="w-16 h-16 mx-auto rounded-2xl bg-purple-100 dark:bg-purple-900/30 flex items-center justify-center mb-3">
                    <Users weight="fill" className="w-8 h-8 text-purple-600 dark:text-purple-400" />
                  </div>
                  <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                    {t('onboarding.steps.users.title')}
                  </h3>
                  <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                    Wie sollen Benutzer hinzugefügt werden?
                  </p>
                </div>
                <div className="grid grid-cols-1 gap-3">
                  <button
                    onClick={() => setSelfRegistration(true)}
                    className={`p-4 rounded-xl border-2 text-left transition-all ${selfRegistration ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20' : 'border-gray-200 dark:border-gray-700 hover:border-gray-300'}`}
                  >
                    <p className="font-medium text-sm">Selbstregistrierung erlauben</p>
                    <p className="text-xs text-gray-400 mt-1">Benutzer können sich selbst registrieren</p>
                  </button>
                  <button
                    onClick={() => setSelfRegistration(false)}
                    className={`p-4 rounded-xl border-2 text-left transition-all ${!selfRegistration ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20' : 'border-gray-200 dark:border-gray-700 hover:border-gray-300'}`}
                  >
                    <p className="font-medium text-sm">Benutzer manuell anlegen</p>
                    <p className="text-xs text-gray-400 mt-1">Nur Admins können Benutzer erstellen</p>
                  </button>
                </div>
              </div>
            )}

            {/* Step 5: Done */}
            {step === 5 && (
              <div className="space-y-4 text-center">
                <div className="w-16 h-16 mx-auto rounded-2xl bg-emerald-100 dark:bg-emerald-900/30 flex items-center justify-center mb-3">
                  <CheckCircle weight="fill" className="w-8 h-8 text-emerald-600 dark:text-emerald-400" />
                </div>
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                  {t('onboarding.steps.done.title')}
                </h3>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  {t('onboarding.steps.done.desc')}
                </p>
                <div className="bg-gray-50 dark:bg-gray-800/50 rounded-xl p-4 text-left space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-500">Passwort:</span>
                    <span className="font-medium text-emerald-600">✓ Geändert</span>
                  </div>
                  {companyName && (
                    <div className="flex justify-between">
                      <span className="text-gray-500">Firma:</span>
                      <span className="font-medium">{companyName}</span>
                    </div>
                  )}
                  <div className="flex justify-between">
                    <span className="text-gray-500">Beispieldaten:</span>
                    <span className="font-medium">{loadDummyData ? 'Ja' : 'Nein'}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-500">Registrierung:</span>
                    <span className="font-medium">{selfRegistration ? 'Selbstregistrierung' : 'Manuell'}</span>
                  </div>
                </div>
              </div>
            )}
          </motion.div>
        </AnimatePresence>

        <div className="flex items-center justify-between mt-6">
          <button
            onClick={() => setStep(Math.max(0, step - 1))}
            disabled={step === 0}
            className="btn btn-secondary btn-sm"
          >
            <ArrowLeft weight="bold" className="w-4 h-4" /> {t('common.back')}
          </button>
          {isLast ? (
            <button onClick={handleNext} className="btn btn-primary">
              {t('onboarding.finish')} <CheckCircle weight="bold" className="w-4 h-4" />
            </button>
          ) : (
            <button onClick={handleNext} disabled={!canGoNext} className="btn btn-primary btn-sm">
              {step === 0 && !passwordChanged ? 'Passwort ändern' : t('common.next')} <ArrowRight weight="bold" className="w-4 h-4" />
            </button>
          )}
        </div>
      </motion.div>
    </div>
  );
}
