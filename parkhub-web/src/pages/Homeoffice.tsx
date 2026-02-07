import { useState, useEffect, useMemo } from 'react';
import { motion } from 'framer-motion';
import { House, Calendar, CalendarCheck, Briefcase, Trash, Plus, CalendarBlank } from '@phosphor-icons/react';
import { api, HomeofficeSettings } from '../api/client';
import toast from 'react-hot-toast';

const WEEKDAYS = ['Montag', 'Dienstag', 'Mittwoch', 'Donnerstag', 'Freitag'];
const WEEKDAY_SHORT = ['Mo', 'Di', 'Mi', 'Do', 'Fr'];

function isSameDay(a: Date, b: Date) {
  return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
}

function getMonday(d: Date) {
  const day = d.getDay();
  const diff = d.getDate() - day + (day === 0 ? -6 : 1);
  return new Date(d.getFullYear(), d.getMonth(), diff);
}

export function HomeofficePage() {
  const [settings, setSettings] = useState<HomeofficeSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [newDate, setNewDate] = useState('');

  useEffect(() => { loadSettings(); }, []);

  async function loadSettings() {
    const res = await api.getHomeofficeSettings();
    if (res.success && res.data) setSettings(res.data);
    setLoading(false);
  }

  const today = new Date();
  const todayDow = today.getDay() === 0 ? 6 : today.getDay() - 1; // 0=Mon

  const isHoToday = useMemo(() => {
    if (!settings) return false;
    if (settings.pattern.weekdays.includes(todayDow)) return true;
    const todayStr = today.toISOString().slice(0, 10);
    return settings.singleDays.some(d => d.date === todayStr);
  }, [settings, todayDow]);

  const hoThisWeek = useMemo(() => {
    if (!settings) return 0;
    const monday = getMonday(today);
    let count = 0;
    for (let i = 0; i < 5; i++) {
      const d = new Date(monday);
      d.setDate(monday.getDate() + i);
      const dStr = d.toISOString().slice(0, 10);
      if (settings.pattern.weekdays.includes(i) || settings.singleDays.some(s => s.date === dStr)) count++;
    }
    return count;
  }, [settings]);

  async function toggleWeekday(day: number) {
    if (!settings) return;
    const current = settings.pattern.weekdays;
    const next = current.includes(day) ? current.filter(d => d !== day) : [...current, day].sort();
    await api.updateHomeofficePattern(next);
    setSettings({ ...settings, pattern: { weekdays: next } });
    toast.success('Homeoffice-Muster aktualisiert');
  }

  async function addDay() {
    if (!newDate) return;
    const res = await api.addHomeofficeDay(newDate);
    if (res.success && res.data && settings) {
      setSettings({ ...settings, singleDays: [...settings.singleDays, res.data] });
      setNewDate('');
      toast.success('Homeoffice-Tag hinzugefügt');
    }
  }

  async function removeDay(id: string) {
    await api.removeHomeofficeDay(id);
    if (settings) {
      setSettings({ ...settings, singleDays: settings.singleDays.filter(d => d.id !== id) });
      toast.success('Homeoffice-Tag entfernt');
    }
  }

  async function addNextWeek() {
    if (!settings) return;
    const nextMon = getMonday(today);
    nextMon.setDate(nextMon.getDate() + 7);
    const newDays = [...settings.singleDays];
    for (let i = 0; i < 5; i++) {
      const d = new Date(nextMon);
      d.setDate(nextMon.getDate() + i);
      const dStr = d.toISOString().slice(0, 10);
      if (!newDays.some(x => x.date === dStr)) {
        const res = await api.addHomeofficeDay(dStr);
        if (res.success && res.data) newDays.push(res.data);
      }
    }
    setSettings({ ...settings, singleDays: newDays });
    toast.success('Nächste Woche als Homeoffice markiert');
  }

  // Calendar
  const calendarDays = useMemo(() => {
    if (!settings) return [];
    const year = today.getFullYear();
    const month = today.getMonth();
    const firstDay = new Date(year, month, 1);
    const lastDay = new Date(year, month + 1, 0);
    const startDow = firstDay.getDay() === 0 ? 6 : firstDay.getDay() - 1;
    const days: { date: Date; inMonth: boolean; isHo: boolean; hoType?: 'pattern' | 'single'; isToday: boolean }[] = [];
    // Fill leading blanks
    for (let i = 0; i < startDow; i++) {
      const d = new Date(year, month, 1 - startDow + i);
      days.push({ date: d, inMonth: false, isHo: false, isToday: false });
    }
    for (let d = 1; d <= lastDay.getDate(); d++) {
      const date = new Date(year, month, d);
      const dow = date.getDay() === 0 ? 6 : date.getDay() - 1;
      const dStr = date.toISOString().slice(0, 10);
      const isPattern = dow < 5 && settings.pattern.weekdays.includes(dow);
      const isSingle = settings.singleDays.some(s => s.date === dStr);
      days.push({
        date, inMonth: true,
        isHo: isPattern || isSingle,
        hoType: isPattern ? 'pattern' : isSingle ? 'single' : undefined,
        isToday: isSameDay(date, today),
      });
    }
    // Fill trailing
    while (days.length % 7 !== 0) {
      const d = new Date(year, month + 1, days.length - startDow - lastDay.getDate() + 1);
      days.push({ date: d, inMonth: false, isHo: false, isToday: false });
    }
    return days;
  }, [settings]);

  const containerVariants = { hidden: { opacity: 0 }, show: { opacity: 1, transition: { staggerChildren: 0.08 } } };
  const itemVariants = { hidden: { opacity: 0, y: 20 }, show: { opacity: 1, y: 0 } };

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="h-8 w-64 skeleton" />
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {[1, 2, 3, 4].map(i => <div key={i} className="h-48 skeleton rounded-2xl" />)}
        </div>
      </div>
    );
  }

  return (
    <motion.div variants={containerVariants} initial="hidden" animate="show" className="space-y-8">
      {/* Header */}
      <motion.div variants={itemVariants}>
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-3">
          <House weight="fill" className="w-7 h-7 text-sky-600" />
          Homeoffice-Verwaltung
        </h1>
        <p className="text-gray-500 dark:text-gray-400 mt-1">
          Verwalten Sie Ihre Homeoffice-Tage und geben Sie Ihren Parkplatz für Kollegen frei.
        </p>
      </motion.div>

      {/* HO Today Banner */}
      {isHoToday && (
        <motion.div variants={itemVariants} className="card bg-sky-50 dark:bg-sky-900/20 border border-sky-200 dark:border-sky-800 p-4">
          <div className="flex items-center gap-3">
            <House weight="fill" className="w-6 h-6 text-sky-600 dark:text-sky-400" />
            <div>
              <p className="font-semibold text-sky-800 dark:text-sky-200">Heute ist Homeoffice-Tag</p>
              <p className="text-sm text-sky-600 dark:text-sky-400">
                Ihr Stellplatz {settings?.parkingSlot?.number} ist für Kollegen freigegeben.
              </p>
            </div>
          </div>
        </motion.div>
      )}

      {/* Status Summary */}
      <motion.div variants={itemVariants} className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="stat-card">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm font-medium text-gray-500 dark:text-gray-400">Diese Woche</p>
              <div className="mt-2 flex items-baseline gap-2">
                <span className="stat-value text-sky-600 dark:text-sky-400">{hoThisWeek}</span>
                <span className="text-gray-500 dark:text-gray-400">Homeoffice-Tage</span>
              </div>
            </div>
            <div className="w-12 h-12 bg-sky-100 dark:bg-sky-900/30 rounded-xl flex items-center justify-center">
              <Calendar weight="fill" className="w-6 h-6 text-sky-600 dark:text-sky-400" />
            </div>
          </div>
        </div>
        {settings?.parkingSlot && (
          <div className="stat-card">
            <div className="flex items-start justify-between">
              <div>
                <p className="text-sm font-medium text-gray-500 dark:text-gray-400">Ihr Parkplatz</p>
                <p className="mt-2 text-sm text-gray-700 dark:text-gray-300">
                  Stellplatz <span className="font-bold text-lg text-gray-900 dark:text-white">{settings.parkingSlot.number}</span> ist an HO-Tagen für Kollegen verfügbar
                </p>
              </div>
              <div className="w-12 h-12 bg-emerald-100 dark:bg-emerald-900/30 rounded-xl flex items-center justify-center">
                <Briefcase weight="fill" className="w-6 h-6 text-emerald-600 dark:text-emerald-400" />
              </div>
            </div>
          </div>
        )}
      </motion.div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Left column */}
        <div className="space-y-8">
          {/* Weekly Pattern */}
          <motion.div variants={itemVariants} className="card p-6">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
              <CalendarCheck weight="fill" className="w-5 h-5 text-sky-600" />
              Regelmäßige Homeoffice-Tage
            </h2>
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-4">
              Wählen Sie die Wochentage, an denen Sie regelmäßig im Homeoffice arbeiten.
            </p>
            <div className="grid grid-cols-5 gap-2">
              {WEEKDAYS.map((name, i) => {
                const active = settings?.pattern.weekdays.includes(i);
                return (
                  <motion.button
                    key={i}
                    whileHover={{ scale: 1.05 }}
                    whileTap={{ scale: 0.95 }}
                    onClick={() => toggleWeekday(i)}
                    className={`flex flex-col items-center gap-1 py-3 px-2 rounded-xl border-2 transition-all font-medium ${
                      active
                        ? 'bg-sky-100 dark:bg-sky-900/40 border-sky-400 dark:border-sky-600 text-sky-700 dark:text-sky-300'
                        : 'bg-gray-50 dark:bg-gray-800 border-gray-200 dark:border-gray-700 text-gray-500 dark:text-gray-400 hover:border-gray-300 dark:hover:border-gray-600'
                    }`}
                  >
                    <span className="text-xs">{WEEKDAY_SHORT[i]}</span>
                    <span className="text-sm">{name}</span>
                    {active && <House weight="fill" className="w-4 h-4 mt-1" />}
                  </motion.button>
                );
              })}
            </div>
          </motion.div>

          {/* Single Days */}
          <motion.div variants={itemVariants} className="card p-6">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
              <CalendarBlank weight="fill" className="w-5 h-5 text-emerald-600" />
              Einzelne Homeoffice-Tage
            </h2>

            <div className="flex gap-2 mb-4">
              <input
                type="date"
                value={newDate}
                onChange={e => setNewDate(e.target.value)}
                className="input flex-1"
                min={today.toISOString().slice(0, 10)}
              />
              <button onClick={addDay} disabled={!newDate} className="btn btn-primary">
                <Plus weight="bold" className="w-4 h-4" />
                Hinzufügen
              </button>
            </div>

            <button onClick={addNextWeek} className="btn btn-ghost text-sm mb-4 w-full justify-center">
              <Calendar weight="bold" className="w-4 h-4" />
              Nächste Woche komplett
            </button>

            <div className="space-y-2 max-h-48 overflow-y-auto">
              {settings?.singleDays
                .filter(d => d.date >= today.toISOString().slice(0, 10))
                .sort((a, b) => a.date.localeCompare(b.date))
                .map(day => (
                  <div key={day.id} className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800/50 rounded-xl">
                    <div className="flex items-center gap-3">
                      <CalendarCheck weight="fill" className="w-5 h-5 text-emerald-500" />
                      <span className="text-sm font-medium text-gray-900 dark:text-white">
                        {new Date(day.date + 'T00:00:00').toLocaleDateString('de-DE', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' })}
                      </span>
                    </div>
                    <button onClick={() => removeDay(day.id)} className="btn btn-ghost btn-icon text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20">
                      <Trash weight="bold" className="w-4 h-4" />
                    </button>
                  </div>
                ))}
              {(!settings?.singleDays || settings.singleDays.filter(d => d.date >= today.toISOString().slice(0, 10)).length === 0) && (
                <p className="text-sm text-gray-400 dark:text-gray-500 text-center py-4">Keine einzelnen HO-Tage geplant</p>
              )}
            </div>
          </motion.div>
        </div>

        {/* Right column - Calendar */}
        <motion.div variants={itemVariants} className="card p-6">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
            <Calendar weight="fill" className="w-5 h-5 text-primary-600" />
            {today.toLocaleDateString('de-DE', { month: 'long', year: 'numeric' })}
          </h2>

          <div className="grid grid-cols-7 gap-1">
            {WEEKDAY_SHORT.map(d => (
              <div key={d} className="text-center text-xs font-semibold text-gray-400 dark:text-gray-500 py-2">{d}</div>
            ))}
            {/* Sa, So */}
            <div className="text-center text-xs font-semibold text-gray-400 dark:text-gray-500 py-2">Sa</div>
            <div className="text-center text-xs font-semibold text-gray-400 dark:text-gray-500 py-2">So</div>

            {calendarDays.map((day, i) => {
              const isWeekend = day.date.getDay() === 0 || day.date.getDay() === 6;
              return (
                <div
                  key={i}
                  className={`aspect-square flex items-center justify-center rounded-lg text-sm font-medium transition-all ${
                    !day.inMonth ? 'text-gray-300 dark:text-gray-700' :
                    day.isToday ? 'ring-2 ring-primary-500 ring-offset-1 dark:ring-offset-gray-900' : ''
                  } ${
                    day.isHo && day.hoType === 'pattern' ? 'bg-sky-200 dark:bg-sky-800/50 text-sky-800 dark:text-sky-200' :
                    day.isHo && day.hoType === 'single' ? 'bg-emerald-200 dark:bg-emerald-800/50 text-emerald-800 dark:text-emerald-200' :
                    isWeekend && day.inMonth ? 'text-gray-400 dark:text-gray-500' :
                    day.inMonth ? 'text-gray-700 dark:text-gray-300' : ''
                  }`}
                >
                  {day.date.getDate()}
                </div>
              );
            })}
          </div>

          {/* Calendar Legend */}
          <div className="flex flex-wrap gap-4 mt-6 pt-4 border-t border-gray-100 dark:border-gray-800 text-xs text-gray-500 dark:text-gray-400">
            <div className="flex items-center gap-1.5">
              <div className="w-3 h-3 rounded-sm bg-sky-200 dark:bg-sky-800/50" />
              <span>Regelmäßig</span>
            </div>
            <div className="flex items-center gap-1.5">
              <div className="w-3 h-3 rounded-sm bg-emerald-200 dark:bg-emerald-800/50" />
              <span>Einzeltag</span>
            </div>
            <div className="flex items-center gap-1.5">
              <div className="w-3 h-3 rounded-sm ring-2 ring-primary-500" />
              <span>Heute</span>
            </div>
          </div>
        </motion.div>
      </div>
    </motion.div>
  );
}
