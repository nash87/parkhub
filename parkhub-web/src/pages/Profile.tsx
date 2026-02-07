import { useState } from 'react';
import { motion } from 'framer-motion';
import {
  User,
  Envelope,
  Shield,
  MapPin,
  CalendarCheck,
  House,
  PencilSimple,
  FloppyDisk,
  ChartBar,
} from '@phosphor-icons/react';
import { useAuth } from '../context/AuthContext';
import toast from 'react-hot-toast';

export function ProfilePage() {
  const { user } = useAuth();
  const [editing, setEditing] = useState(false);
  const [formData, setFormData] = useState({ name: user?.name || '', email: user?.email || '' });

  function handleSave() {
    setEditing(false);
    toast.success('Profil aktualisiert');
  }

  const initials = user?.name?.split(' ').map(n => n[0]).join('').toUpperCase() || '?';
  const roleLabels: Record<string, string> = { user: 'Benutzer', admin: 'Administrator', superadmin: 'Super-Admin' };

  const containerVariants = { hidden: { opacity: 0 }, show: { opacity: 1, transition: { staggerChildren: 0.1 } } };
  const itemVariants = { hidden: { opacity: 0, y: 20 }, show: { opacity: 1, y: 0 } };

  return (
    <motion.div variants={containerVariants} initial="hidden" animate="show" className="max-w-3xl mx-auto space-y-8">
      <motion.div variants={itemVariants}>
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Mein Profil</h1>
        <p className="text-gray-500 dark:text-gray-400 mt-1">Ihre persönlichen Daten und Statistiken</p>
      </motion.div>

      {/* User Info Card */}
      <motion.div variants={itemVariants} className="card p-8">
        <div className="flex flex-col sm:flex-row items-center sm:items-start gap-6">
          <div className="w-24 h-24 rounded-2xl bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center flex-shrink-0">
            <span className="text-3xl font-bold text-primary-600 dark:text-primary-400">{initials}</span>
          </div>
          <div className="flex-1 text-center sm:text-left">
            {editing ? (
              <div className="space-y-3">
                <div>
                  <label className="label">Name</label>
                  <input
                    type="text"
                    value={formData.name}
                    onChange={e => setFormData({ ...formData, name: e.target.value })}
                    className="input"
                  />
                </div>
                <div>
                  <label className="label">E-Mail</label>
                  <input
                    type="email"
                    value={formData.email}
                    onChange={e => setFormData({ ...formData, email: e.target.value })}
                    className="input"
                  />
                </div>
                <div className="flex gap-2">
                  <button onClick={handleSave} className="btn btn-primary btn-sm">
                    <FloppyDisk weight="bold" className="w-4 h-4" /> Speichern
                  </button>
                  <button onClick={() => setEditing(false)} className="btn btn-secondary btn-sm">Abbrechen</button>
                </div>
              </div>
            ) : (
              <>
                <h2 className="text-xl font-bold text-gray-900 dark:text-white">{user?.name}</h2>
                <div className="flex flex-wrap items-center justify-center sm:justify-start gap-3 mt-2">
                  <span className="flex items-center gap-1.5 text-sm text-gray-500 dark:text-gray-400">
                    <User weight="regular" className="w-4 h-4" />
                    @{user?.username}
                  </span>
                  <span className="flex items-center gap-1.5 text-sm text-gray-500 dark:text-gray-400">
                    <Envelope weight="regular" className="w-4 h-4" />
                    {user?.email}
                  </span>
                </div>
                <div className="mt-3 flex flex-wrap items-center justify-center sm:justify-start gap-2">
                  <span className={`badge ${user?.role === 'admin' || user?.role === 'superadmin' ? 'badge-warning' : 'badge-info'}`}>
                    <Shield weight="fill" className="w-3 h-3" />
                    {roleLabels[user?.role || 'user']}
                  </span>
                  <button onClick={() => setEditing(true)} className="btn btn-ghost btn-sm">
                    <PencilSimple weight="bold" className="w-3.5 h-3.5" /> Bearbeiten
                  </button>
                </div>
              </>
            )}
          </div>
        </div>
      </motion.div>

      {/* Mein Stellplatz */}
      <motion.div variants={itemVariants} className="card p-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
          <MapPin weight="fill" className="w-5 h-5 text-primary-600" />
          Mein Stellplatz
        </h3>
        <div className="flex items-center gap-4 p-4 bg-gray-50 dark:bg-gray-800/50 rounded-xl">
          <div className="w-16 h-16 bg-primary-100 dark:bg-primary-900/30 rounded-2xl flex items-center justify-center">
            <span className="text-2xl font-bold text-primary-600 dark:text-primary-400">47</span>
          </div>
          <div>
            <p className="font-semibold text-gray-900 dark:text-white">Firmenparkplatz</p>
            <p className="text-sm text-gray-500 dark:text-gray-400">Hauptstraße 1 · Reihe A</p>
            <p className="text-xs text-gray-400 mt-1">Fester Stellplatz · An HO-Tagen freigegeben</p>
          </div>
        </div>
      </motion.div>

      {/* Statistics */}
      <motion.div variants={itemVariants} className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div className="stat-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Buchungen diesen Monat</p>
              <p className="stat-value text-primary-600 dark:text-primary-400 mt-1">12</p>
            </div>
            <CalendarCheck weight="fill" className="w-8 h-8 text-primary-200 dark:text-primary-800" />
          </div>
        </div>
        <div className="stat-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Homeoffice-Tage</p>
              <p className="stat-value text-sky-600 dark:text-sky-400 mt-1">8</p>
            </div>
            <House weight="fill" className="w-8 h-8 text-sky-200 dark:text-sky-800" />
          </div>
        </div>
        <div className="stat-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Durchschn. Parkdauer</p>
              <p className="stat-value text-amber-600 dark:text-amber-400 mt-1">6.2h</p>
            </div>
            <ChartBar weight="fill" className="w-8 h-8 text-amber-200 dark:text-amber-800" />
          </div>
        </div>
      </motion.div>
    </motion.div>
  );
}
