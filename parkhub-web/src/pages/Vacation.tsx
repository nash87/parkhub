import { useState, useEffect, useMemo, useRef } from 'react';
import { motion } from 'framer-motion';
import { Airplane, Calendar, Trash, Plus, CaretLeft, CaretRight, UploadSimple, UsersThree } from '@phosphor-icons/react';
import { api, VacationEntry, TeamVacationEntry } from '../api/client';
import { useTranslation } from 'react-i18next';
import toast from 'react-hot-toast';

function isSameDay(a: Date, b: Date) { return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate(); }

function isDateInRange(date: Date, start: string, end: string) {
  const d = date.toISOString().slice(0, 10);
  return d >= start && d <= end;
}

export function VacationPage() {
  const { t } = useTranslation();
  const [entries, setEntries] = useState<VacationEntry[]>([]);
  const [teamEntries, setTeamEntries] = useState<TeamVacationEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');
  const [note, setNote] = useState('');
  const fileRef = useRef<HTMLInputElement>(null);

  const today = new Date();

  useEffect(() => { loadData(); }, []);

  async function loadData() {
    try {
      const [vacRes, teamRes] = await Promise.all([
        api.listVacation(),
        api.teamVacation(),
      ]);
      if (vacRes.success && vacRes.data) setEntries(vacRes.data);
      if (teamRes.success && teamRes.data) setTeamEntries(teamRes.data);
    } catch {}
    setLoading(false);
  }

  async function addVacation() {
    if (!startDate || !endDate) return;
    const res = await api.createVacation(startDate, endDate, note || undefined);
    if (res.success && res.data) {
      setEntries(prev => [...prev, res.data!].sort((a, b) => a.start_date.localeCompare(b.start_date)));
      setStartDate(''); setEndDate(''); setNote('');
      toast.success(t('vacation.added'));
    } else {
      toast.error(res.error?.message || 'Error');
    }
  }

  async function removeVacation(id: string) {
    const res = await api.deleteVacation(id);
    if (res.success) {
      setEntries(prev => prev.filter(e => e.id !== id));
      toast.success(t('vacation.removed'));
    }
  }

  async function importIcal() {
    fileRef.current?.click();
  }

  async function handleFileUpload(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    try {
      const data = await api.importVacationIcal(file);
      if (data.success && data.data) {
        setEntries(prev => [...prev, ...(data.data || [])].sort((a: VacationEntry, b: VacationEntry) => a.start_date.localeCompare(b.start_date)));
        toast.success(t('vacation.imported', { count: data.data.length }));
      } else {
        toast.error(data.error?.message || 'Import failed');
      }
    } catch {
      toast.error('Import failed');
    }
    if (fileRef.current) fileRef.current.value = '';
  }

  // Calendar
  const [calMonth, setCalMonth] = useState(today.getMonth());
  const [calYear, setCalYear] = useState(today.getFullYear());
  function prevMonth() { if (calMonth === 0) { setCalMonth(11); setCalYear(y => y - 1); } else setCalMonth(m => m - 1); }
  function nextMonth() { if (calMonth === 11) { setCalMonth(0); setCalYear(y => y + 1); } else setCalMonth(m => m + 1); }

  const WEEKDAY_SHORT = [t('homeoffice.weekdaysShort.mon'), t('homeoffice.weekdaysShort.tue'), t('homeoffice.weekdaysShort.wed'), t('homeoffice.weekdaysShort.thu'), t('homeoffice.weekdaysShort.fri'), t('homeoffice.weekdaysShort.sat'), t('homeoffice.weekdaysShort.sun')];

  const calendarDays = useMemo(() => {
    const year = calYear; const month = calMonth;
    const firstDay = new Date(year, month, 1); const lastDay = new Date(year, month + 1, 0);
    const startDow = firstDay.getDay() === 0 ? 6 : firstDay.getDay() - 1;
    const days: { date: Date; inMonth: boolean; isVacation: boolean; isToday: boolean }[] = [];
    for (let i = 0; i < startDow; i++) { const d = new Date(year, month, 1 - startDow + i); days.push({ date: d, inMonth: false, isVacation: false, isToday: false }); }
    for (let d = 1; d <= lastDay.getDate(); d++) {
      const date = new Date(year, month, d);
      const isVac = entries.some(e => isDateInRange(date, e.start_date, e.end_date));
      days.push({ date, inMonth: true, isVacation: isVac, isToday: isSameDay(date, today) });
    }
    while (days.length % 7 !== 0) { const d = new Date(year, month + 1, days.length - startDow - lastDay.getDate() + 1); days.push({ date: d, inMonth: false, isVacation: false, isToday: false }); }
    return days;
  }, [entries, calMonth, calYear]);

  const calMonthLabel = new Date(calYear, calMonth, 1).toLocaleDateString(undefined, { month: 'long', year: 'numeric' });

  // Count vacation days this month
  const vacDaysThisMonth = useMemo(() => {
    let count = 0;
    const year = today.getFullYear(); const month = today.getMonth();
    const lastDay = new Date(year, month + 1, 0).getDate();
    for (let d = 1; d <= lastDay; d++) {
      const date = new Date(year, month, d);
      if (entries.some(e => isDateInRange(date, e.start_date, e.end_date))) count++;
    }
    return count;
  }, [entries]);

  const isOnVacationToday = entries.some(e => isDateInRange(today, e.start_date, e.end_date));

  const containerVariants = { hidden: { opacity: 0 }, show: { opacity: 1, transition: { staggerChildren: 0.08 } } };
  const itemVariants = { hidden: { opacity: 0, y: 20 }, show: { opacity: 1, y: 0 } };

  if (loading) return <div className="space-y-6"><div className="h-8 w-64 skeleton" /><div className="grid grid-cols-1 md:grid-cols-2 gap-6">{[1,2,3,4].map(i => <div key={i} className="h-48 skeleton rounded-2xl" />)}</div></div>;

  return (
    <motion.div variants={containerVariants} initial="hidden" animate="show" className="space-y-8">
      <motion.div variants={itemVariants}>
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-3"><Airplane weight="fill" className="w-7 h-7 text-orange-600" />{t('vacation.title')}</h1>
        <p className="text-gray-500 dark:text-gray-400 mt-1">{t('vacation.subtitle')}</p>
      </motion.div>

      {isOnVacationToday && (
        <motion.div variants={itemVariants} className="card bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800 p-4">
          <div className="flex items-center gap-3"><Airplane weight="fill" className="w-6 h-6 text-orange-600 dark:text-orange-400" /><div>
            <p className="font-semibold text-orange-800 dark:text-orange-200">{t('vacation.todayBanner')}</p>
            <p className="text-sm text-orange-600 dark:text-orange-400">{t('vacation.todayBannerDesc')}</p>
          </div></div>
        </motion.div>
      )}

      <motion.div variants={itemVariants} className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="stat-card">
          <div className="flex items-start justify-between"><div><p className="text-sm font-medium text-gray-500 dark:text-gray-400">{t('vacation.thisMonth')}</p><div className="mt-2 flex items-baseline gap-2"><span className="stat-value text-orange-600 dark:text-orange-400">{vacDaysThisMonth}</span><span className="text-gray-500 dark:text-gray-400">{t('vacation.vacationDays')}</span></div></div>
          <div className="w-12 h-12 bg-orange-100 dark:bg-orange-900/30 rounded-xl flex items-center justify-center"><Calendar weight="fill" className="w-6 h-6 text-orange-600 dark:text-orange-400" /></div></div>
        </div>
        <div className="stat-card">
          <div className="flex items-start justify-between"><div><p className="text-sm font-medium text-gray-500 dark:text-gray-400">{t('vacation.totalEntries')}</p><div className="mt-2 flex items-baseline gap-2"><span className="stat-value text-orange-600 dark:text-orange-400">{entries.length}</span><span className="text-gray-500 dark:text-gray-400">{t('vacation.entries')}</span></div></div>
          <div className="w-12 h-12 bg-orange-100 dark:bg-orange-900/30 rounded-xl flex items-center justify-center"><Airplane weight="fill" className="w-6 h-6 text-orange-600 dark:text-orange-400" /></div></div>
        </div>
      </motion.div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <div className="space-y-8">
          <motion.div variants={itemVariants} className="card p-6">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2"><Plus weight="bold" className="w-5 h-5 text-orange-600" />{t('vacation.addVacation')}</h2>
            <div className="space-y-3">
              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="text-xs text-gray-500 dark:text-gray-400">{t('vacation.startDate')}</label>
                  <input type="date" value={startDate} onChange={e => setStartDate(e.target.value)} className="input w-full" min={today.toISOString().slice(0, 10)} />
                </div>
                <div>
                  <label className="text-xs text-gray-500 dark:text-gray-400">{t('vacation.endDate')}</label>
                  <input type="date" value={endDate} onChange={e => setEndDate(e.target.value)} className="input w-full" min={startDate || today.toISOString().slice(0, 10)} />
                </div>
              </div>
              <input type="text" placeholder={t('vacation.notePlaceholder')} value={note} onChange={e => setNote(e.target.value)} className="input w-full" />
              <button onClick={addVacation} disabled={!startDate || !endDate} className="btn btn-primary w-full"><Plus weight="bold" className="w-4 h-4" />{t('vacation.addBtn')}</button>
            </div>
          </motion.div>

          <motion.div variants={itemVariants} className="card p-6">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2"><UploadSimple weight="bold" className="w-5 h-5 text-orange-600" />{t('vacation.importIcal')}</h2>
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-3">{t('vacation.importDesc')}</p>
            <input ref={fileRef} type="file" accept=".ics,.ical" onChange={handleFileUpload} className="hidden" />
            <button onClick={importIcal} className="btn btn-ghost w-full justify-center"><UploadSimple weight="bold" className="w-4 h-4" />{t('vacation.importBtn')}</button>
          </motion.div>

          <motion.div variants={itemVariants} className="card p-6">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2"><Airplane weight="fill" className="w-5 h-5 text-orange-600" />{t('vacation.myVacations')}</h2>
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {entries.filter(e => e.end_date >= today.toISOString().slice(0, 10)).map(entry => (
                <div key={entry.id} className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800/50 rounded-xl">
                  <div className="flex items-center gap-3">
                    <Airplane weight="fill" className="w-5 h-5 text-orange-500" />
                    <div>
                      <span className="text-sm font-medium text-gray-900 dark:text-white">
                        {new Date(entry.start_date + 'T00:00:00').toLocaleDateString(undefined, { day: 'numeric', month: 'short' })}
                        {entry.start_date !== entry.end_date && <> – {new Date(entry.end_date + 'T00:00:00').toLocaleDateString(undefined, { day: 'numeric', month: 'short', year: 'numeric' })}</>}
                        {entry.start_date === entry.end_date && <>, {new Date(entry.start_date + 'T00:00:00').getFullYear()}</>}
                      </span>
                      {entry.note && <p className="text-xs text-gray-500 dark:text-gray-400">{entry.note}</p>}
                    </div>
                  </div>
                  <button onClick={() => removeVacation(entry.id)} className="btn btn-ghost btn-icon text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20"><Trash weight="bold" className="w-4 h-4" /></button>
                </div>
              ))}
              {entries.filter(e => e.end_date >= today.toISOString().slice(0, 10)).length === 0 && <p className="text-sm text-gray-400 dark:text-gray-500 text-center py-4">{t('vacation.noEntries')}</p>}
            </div>
          </motion.div>
        </div>

        <div className="space-y-8">
          <motion.div variants={itemVariants} className="card p-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2"><Calendar weight="fill" className="w-5 h-5 text-primary-600" />{calMonthLabel}</h2>
              <div className="flex items-center gap-1">
                <button onClick={prevMonth} className="btn btn-ghost btn-icon" aria-label="Previous month"><CaretLeft weight="bold" className="w-5 h-5" /></button>
                <button onClick={nextMonth} className="btn btn-ghost btn-icon" aria-label="Next month"><CaretRight weight="bold" className="w-5 h-5" /></button>
              </div>
            </div>
            <div className="grid grid-cols-7 gap-1">
              {WEEKDAY_SHORT.map(d => <div key={d} className="text-center text-xs font-semibold text-gray-400 dark:text-gray-500 py-2">{d}</div>)}
              {calendarDays.map((day, i) => {
                const isWeekend = day.date.getDay() === 0 || day.date.getDay() === 6;
                return (
                  <div key={i} className={`aspect-square flex items-center justify-center rounded-lg text-sm font-medium transition-all ${!day.inMonth ? 'text-gray-300 dark:text-gray-700' : day.isToday ? 'ring-2 ring-primary-500 ring-offset-1 dark:ring-offset-gray-900' : ''} ${day.isVacation ? 'bg-orange-200 dark:bg-orange-800/50 text-orange-800 dark:text-orange-200' : isWeekend && day.inMonth ? 'text-gray-400 dark:text-gray-500' : day.inMonth ? 'text-gray-700 dark:text-gray-300' : ''}`}>{day.date.getDate()}</div>
                );
              })}
            </div>
            <div className="flex flex-wrap gap-4 mt-6 pt-4 border-t border-gray-100 dark:border-gray-800 text-xs text-gray-500 dark:text-gray-400">
              <div className="flex items-center gap-1.5"><div className="w-3 h-3 rounded-sm bg-orange-200 dark:bg-orange-800/50" /><span>{t('vacation.legendVacation')}</span></div>
              <div className="flex items-center gap-1.5"><div className="w-3 h-3 rounded-sm ring-2 ring-primary-500" /><span>{t('vacation.legendToday')}</span></div>
            </div>
          </motion.div>

          <motion.div variants={itemVariants} className="card p-6">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2"><UsersThree weight="fill" className="w-5 h-5 text-orange-600" />{t('vacation.teamOverview')}</h2>
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {teamEntries.filter(e => e.end_date >= today.toISOString().slice(0, 10)).map((entry, i) => (
                <div key={i} className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800/50 rounded-xl">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 bg-orange-100 dark:bg-orange-900/30 rounded-full flex items-center justify-center text-xs font-bold text-orange-700 dark:text-orange-300">{entry.user_name.charAt(0).toUpperCase()}</div>
                    <div>
                      <p className="text-sm font-medium text-gray-900 dark:text-white">{entry.user_name}</p>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        {new Date(entry.start_date + 'T00:00:00').toLocaleDateString(undefined, { day: 'numeric', month: 'short' })}
                        {entry.start_date !== entry.end_date && <> – {new Date(entry.end_date + 'T00:00:00').toLocaleDateString(undefined, { day: 'numeric', month: 'short' })}</>}
                      </p>
                    </div>
                  </div>
                </div>
              ))}
              {teamEntries.filter(e => e.end_date >= today.toISOString().slice(0, 10)).length === 0 && <p className="text-sm text-gray-400 dark:text-gray-500 text-center py-4">{t('vacation.noTeamEntries')}</p>}
            </div>
          </motion.div>
        </div>
      </div>
    </motion.div>
  );
}
