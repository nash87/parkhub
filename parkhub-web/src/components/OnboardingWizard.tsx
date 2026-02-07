import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Lock, Buildings, Car, SquaresFour, Users, CheckCircle, ArrowRight, ArrowLeft, X } from '@phosphor-icons/react';
import { useTranslation } from 'react-i18next';

interface OnboardingWizardProps {
  onComplete: () => void;
}

const steps = [
  { icon: Lock, key: 'password' },
  { icon: Buildings, key: 'company' },
  { icon: Car, key: 'lot' },
  { icon: SquaresFour, key: 'slots' },
  { icon: Users, key: 'users' },
  { icon: CheckCircle, key: 'done' },
];

export function OnboardingWizard({ onComplete }: OnboardingWizardProps) {
  const { t } = useTranslation();
  const [step, setStep] = useState(0);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (!localStorage.getItem('parkhub_onboarding_done')) setVisible(true);
  }, []);

  function finish() {
    localStorage.setItem('parkhub_onboarding_done', 'true');
    setVisible(false);
    onComplete();
  }

  function skip() {
    localStorage.setItem('parkhub_onboarding_done', 'true');
    setVisible(false);
  }

  if (!visible) return null;

  const Icon = steps[step].icon;
  const isLast = step === steps.length - 1;

  return (
    <div className="fixed inset-0 z-[70] flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
      <motion.div
        initial={{ scale: 0.9, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        className="card w-full max-w-lg p-8 shadow-2xl"
      >
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-bold text-gray-900 dark:text-white">{t('onboarding.title')}</h2>
          <button onClick={skip} className="btn btn-ghost btn-icon p-1" aria-label={t('common.close')}>
            <X weight="bold" className="w-5 h-5" />
          </button>
        </div>

        {/* Progress */}
        <div className="flex gap-1 mb-8">
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
            className="text-center py-6"
          >
            <div className="w-16 h-16 mx-auto rounded-2xl bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center mb-4">
              <Icon weight="fill" className="w-8 h-8 text-primary-600 dark:text-primary-400" />
            </div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
              {t(`onboarding.steps.${steps[step].key}.title`)}
            </h3>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              {t(`onboarding.steps.${steps[step].key}.desc`)}
            </p>
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
            <button onClick={finish} className="btn btn-primary btn-sm">
              {t('onboarding.finish')} <CheckCircle weight="bold" className="w-4 h-4" />
            </button>
          ) : (
            <button onClick={() => setStep(step + 1)} className="btn btn-primary btn-sm">
              {t('common.next')} <ArrowRight weight="bold" className="w-4 h-4" />
            </button>
          )}
        </div>
      </motion.div>
    </div>
  );
}
