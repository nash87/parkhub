import { motion } from 'framer-motion';
import { Scales } from '@phosphor-icons/react';
import { useTranslation } from 'react-i18next';

export function LegalPage() {
  const { t } = useTranslation();

  return (
    <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="max-w-3xl mx-auto space-y-8 py-8 px-4">
      <div className="flex items-center gap-3">
        <Scales weight="fill" className="w-8 h-8 text-primary-600" />
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">{t('legal.title')}</h1>
      </div>
      <div className="card p-6">
        <p className="text-sm text-gray-600 dark:text-gray-400 leading-relaxed whitespace-pre-line">{t('legal.content')}</p>
      </div>
    </motion.div>
  );
}
