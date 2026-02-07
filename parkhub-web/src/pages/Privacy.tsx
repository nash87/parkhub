import { motion } from 'framer-motion';
import { Shield, Database, Lock, Eye, Server } from '@phosphor-icons/react';
import { useTranslation } from 'react-i18next';

export function PrivacyPage() {
  const { t } = useTranslation();
  const sections = [
    { icon: Database, key: 'dataCollected' },
    { icon: Server, key: 'storage' },
    { icon: Lock, key: 'security' },
    { icon: Eye, key: 'access' },
  ];

  return (
    <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="max-w-3xl mx-auto space-y-8 py-8 px-4">
      <div>
        <div className="flex items-center gap-3 mb-2">
          <Shield weight="fill" className="w-8 h-8 text-primary-600" />
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">{t('privacy.title')}</h1>
        </div>
        <p className="text-gray-500 dark:text-gray-400">{t('privacy.subtitle')}</p>
      </div>

      {sections.map(({ icon: Icon, key }) => (
        <div key={key} className="card p-6">
          <div className="flex items-center gap-3 mb-3">
            <Icon weight="fill" className="w-5 h-5 text-primary-600" />
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">{t(`privacy.${key}.title`)}</h2>
          </div>
          <p className="text-sm text-gray-600 dark:text-gray-400 leading-relaxed whitespace-pre-line">{t(`privacy.${key}.content`)}</p>
        </div>
      ))}

      <div className="card p-6">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-3">{t('privacy.rights.title')}</h2>
        <ul className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
          {['access', 'rectification', 'erasure', 'portability'].map(r => (
            <li key={r} className="flex items-start gap-2">
              <span className="text-primary-600 mt-0.5">•</span>
              <span>{t(`privacy.rights.${r}`)}</span>
            </li>
          ))}
        </ul>
      </div>
    </motion.div>
  );
}
