import { useEffect, useState } from 'react';
import { Routes, Route, Link, useLocation } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';
import {
  ChartBar, Buildings, Users, ListChecks, Plus, CheckCircle, TrendUp, CaretRight,
  SpinnerGap, MagnifyingGlass, XCircle, Trash, PencilSimple,
  Lightning, Pulse, ShieldCheck, Clock, House, Prohibit, Palette,
} from '@phosphor-icons/react';
import { api, ParkingLot, ParkingLotDetailed } from '../api/client';
import { LotLayoutEditor } from '../components/LotLayoutEditor';
import { AdminBrandingPage } from './AdminBranding';
import { useTranslation } from 'react-i18next';

function AdminNav() {
  const { t } = useTranslation();
  const location = useLocation();
  const tabs = [
    { name: t('admin.tabs.overview'), path: '/admin', icon: ChartBar },
    { name: t('admin.tabs.lots'), path: '/admin/lots', icon: Buildings },
    { name: t('admin.tabs.users'), path: '/admin/users', icon: Users },
    { name: t('admin.tabs.bookings'), path: '/admin/bookings', icon: ListChecks },
    { name: t('admin.tabs.branding', 'Branding'), path: '/admin/branding', icon: Palette },
  ];
  return (
    <div className="border-b border-gray-200 dark:border-gray-800 mb-8">
      <nav className="flex gap-1 overflow-x-auto">
        {tabs.map((tab) => {
          const Icon = tab.icon;
          const isActive = location.pathname === tab.path;
          return <Link key={tab.path} to={tab.path} className={`flex items-center gap-2 px-4 py-3 text-sm font-medium whitespace-nowrap border-b-2 transition-colors ${isActive ? 'border-primary-600 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300'}`}><Icon weight={isActive ? 'fill' : 'regular'} className="w-5 h-5" />{tab.name}</Link>;
        })}
      </nav>
    </div>
  );
}

function AdminOverview() {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(true);
  useEffect(() => { const tm = setTimeout(() => setLoading(false), 400); return () => clearTimeout(tm); }, []);

  const stats = [
    { label: t('admin.overview.totalSlots'), value: '37', icon: Buildings, color: 'bg-blue-100 dark:bg-blue-900/30', iconColor: 'text-blue-600 dark:text-blue-400' },
    { label: t('admin.overview.activeBookings'), value: '14', icon: Clock, color: 'bg-emerald-100 dark:bg-emerald-900/30', iconColor: 'text-emerald-600 dark:text-emerald-400' },
    { label: t('admin.overview.occupancyToday'), value: '38%', icon: TrendUp, color: 'bg-amber-100 dark:bg-amber-900/30', iconColor: 'text-amber-600 dark:text-amber-400' },
    { label: t('admin.overview.homeofficeToday'), value: '5', icon: House, color: 'bg-purple-100 dark:bg-purple-900/30', iconColor: 'text-purple-600 dark:text-purple-400' },
  ];

  const recentActivity = [
    { text: 'Max M. hat Stellplatz 47 gebucht', time: 'vor 5 Min', type: 'booking' as const },
    { text: 'Lisa K. hat Homeoffice für morgen aktiviert', time: 'vor 12 Min', type: 'homeoffice' as const },
    { text: 'Thomas B. hat Buchung für Stellplatz 12 storniert', time: 'vor 25 Min', type: 'cancel' as const },
    { text: 'Anna S. hat neues Fahrzeug registriert (M-AS 4521)', time: 'vor 1 Std', type: 'vehicle' as const },
  ];
  const activityIcons = { booking: { icon: CheckCircle, color: 'text-emerald-500' }, homeoffice: { icon: House, color: 'text-purple-500' }, cancel: { icon: XCircle, color: 'text-red-500' }, vehicle: { icon: Plus, color: 'text-blue-500' } };
  const systemStatus = [{ name: t('admin.overview.backendApi'), status: 'online' }, { name: t('admin.overview.database'), status: 'online' }, { name: t('admin.overview.authService'), status: 'online' }];

  if (loading) return <div className="space-y-8"><div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">{[1,2,3,4].map(i => <div key={i} className="h-32 skeleton rounded-2xl" />)}</div><div className="h-64 skeleton rounded-2xl" /></div>;

  return (
    <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="space-y-8">
      <h2 className="text-xl font-semibold text-gray-900 dark:text-white">{t('admin.overview.title')}</h2>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {stats.map((s, i) => { const Icon = s.icon; return (
          <motion.div key={s.label} initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: i * 0.08 }} className="stat-card">
            <div className="flex items-center justify-between"><div><p className="stat-label">{s.label}</p><p className="stat-value text-gray-900 dark:text-white">{s.value}</p></div><div className={`w-12 h-12 ${s.color} rounded-xl flex items-center justify-center`}><Icon weight="fill" className={`w-6 h-6 ${s.iconColor}`} /></div></div>
          </motion.div>
        ); })}
      </div>
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 card p-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2"><Pulse weight="fill" className="w-5 h-5 text-primary-600" />{t('admin.overview.recentActivity')}</h3>
          <div className="space-y-3">{recentActivity.map((a, i) => { const ac = activityIcons[a.type]; const AIcon = ac.icon; return (
            <motion.div key={i} initial={{ opacity: 0, x: -10 }} animate={{ opacity: 1, x: 0 }} transition={{ delay: i * 0.08 }} className="flex items-center gap-3 p-3 rounded-xl bg-gray-50 dark:bg-gray-800/50">
              <AIcon weight="fill" className={`w-5 h-5 flex-shrink-0 ${ac.color}`} /><p className="text-sm text-gray-700 dark:text-gray-300 flex-1">{a.text}</p><span className="text-xs text-gray-400 whitespace-nowrap">{a.time}</span>
            </motion.div>
          ); })}</div>
        </div>
        <div className="space-y-6">
          <div className="card p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2"><Lightning weight="fill" className="w-5 h-5 text-amber-500" />{t('admin.overview.quickActions')}</h3>
            <div className="space-y-2">
              <button className="btn btn-secondary w-full justify-start"><Prohibit weight="bold" className="w-4 h-4" />{t('admin.overview.blockSlot')}</button>
              <Link to="/admin/users" className="btn btn-secondary w-full justify-start"><Users weight="regular" className="w-4 h-4" />{t('admin.overview.manageUsers')}</Link>
              <button className="btn btn-secondary w-full justify-start"><XCircle weight="bold" className="w-4 h-4" />{t('admin.overview.cancelBooking')}</button>
            </div>
          </div>
          <div className="card p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2"><ShieldCheck weight="fill" className="w-5 h-5 text-emerald-500" />{t('admin.overview.systemStatus')}</h3>
            <div className="space-y-3">{systemStatus.map((s) => (
              <div key={s.name} className="flex items-center justify-between"><span className="text-sm text-gray-700 dark:text-gray-300">{s.name}</span><span className="flex items-center gap-1.5 text-xs font-medium text-emerald-600 dark:text-emerald-400"><CheckCircle weight="fill" className="w-4 h-4" />{t('common.online')}</span></div>
            ))}</div>
          </div>
        </div>
      </div>
    </motion.div>
  );
}

function AdminLots() {
  const { t } = useTranslation();
  const [lots, setLots] = useState<ParkingLot[]>([]);
  const [loading, setLoading] = useState(true);
  const [editingLotId, setEditingLotId] = useState<string | null>(null);
  const [editingLayout, setEditingLayout] = useState<ParkingLotDetailed | null>(null);
  const [showNewEditor, setShowNewEditor] = useState(false);

  useEffect(() => { loadLots(); }, []);
  async function loadLots() { try { const res = await api.getLots(); if (res.success && res.data) setLots(res.data); } finally { setLoading(false); } }
  const [deletingLotId, setDeletingLotId] = useState<string | null>(null);
  async function handleDeleteLot(lotId: string, _lotName: string) {
    if (!confirm(t('admin.lots.confirmDelete', 'Parkplatz  + lotName +  wirklich löschen? Alle zugehörigen Stellplätze und Buchungen werden ebenfalls gelöscht.'))) return;
    setDeletingLotId(lotId);
    try {
      const token = (window as any).__parkhub_token || localStorage.getItem('parkhub_token');
      const res = await fetch('/api/v1/admin/lots/' + lotId, {
        method: 'DELETE',
        headers: { 'Authorization': 'Bearer ' + token },
      });
      const data = await res.json();
      if (data.success) {
        setLots(prev => prev.filter(l => l.id !== lotId));
        if (editingLotId === lotId) { setEditingLotId(null); setEditingLayout(null); }
      }
    } finally { setDeletingLotId(null); }
  }
  async function handleEdit(lot: ParkingLot) {
    if (editingLotId === lot.id) { setEditingLotId(null); setEditingLayout(null); return; }
    const res = await api.getLotDetailed(lot.id);
    if (res.success && res.data) { setEditingLotId(lot.id); setEditingLayout(res.data); setShowNewEditor(false); }
  }

  if (loading) return <div className="flex items-center justify-center h-64"><SpinnerGap weight="bold" className="w-8 h-8 text-primary-600 animate-spin" /></div>;

  return (
    <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white">{t('admin.lots.title')}</h2>
        <button onClick={() => { setShowNewEditor(p => !p); setEditingLotId(null); }} className="btn btn-primary"><Plus weight="bold" className="w-4 h-4" />{t('admin.lots.newLot')}</button>
      </div>
      <AnimatePresence>{showNewEditor && (
        <motion.div initial={{ opacity: 0, height: 0 }} animate={{ opacity: 1, height: 'auto' }} exit={{ opacity: 0, height: 0 }} className="overflow-hidden">
          <div className="card p-6"><h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">{t('admin.lots.createLot')}</h3><LotLayoutEditor onSave={() => setShowNewEditor(false)} onCancel={() => setShowNewEditor(false)} /></div>
        </motion.div>
      )}</AnimatePresence>
      <div className="space-y-4">{lots.map((lot) => (
        <div key={lot.id}>
          <div className="card-hover p-6">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4"><div className="w-12 h-12 bg-gray-100 dark:bg-gray-800 rounded-xl flex items-center justify-center"><Buildings weight="fill" className="w-6 h-6 text-gray-500" /></div><div><p className="font-semibold text-gray-900 dark:text-white">{lot.name}</p><p className="text-sm text-gray-500 dark:text-gray-400">{lot.address}</p></div></div>
              <div className="flex items-center gap-4"><div className="text-right"><p className="font-bold text-gray-900 dark:text-white">{lot.available_slots}/{lot.total_slots}</p><p className="text-xs text-gray-500">{t('common.available')}</p></div>
              <button onClick={() => handleDeleteLot(lot.id, lot.name)} disabled={deletingLotId === lot.id} className="btn btn-sm btn-ghost text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20"><Trash weight="regular" className="w-4 h-4" /></button>
              <button onClick={() => handleEdit(lot)} className={`btn btn-sm ${editingLotId === lot.id ? 'btn-primary' : 'btn-secondary'}`}>{t('admin.lots.edit')}<CaretRight weight="bold" className={`w-4 h-4 transition-transform ${editingLotId === lot.id ? 'rotate-90' : ''}`} /></button></div>
            </div>
          </div>
          <AnimatePresence>{editingLotId === lot.id && editingLayout && (
            <motion.div initial={{ opacity: 0, height: 0 }} animate={{ opacity: 1, height: 'auto' }} exit={{ opacity: 0, height: 0 }} className="overflow-hidden">
              <div className="card p-6 mt-2 border-l-4 border-l-primary-500"><LotLayoutEditor initialLayout={editingLayout.layout} lotName={editingLayout.name} onSave={() => setEditingLotId(null)} onCancel={() => setEditingLotId(null)} /></div>
            </motion.div>
          )}</AnimatePresence>
        </div>
      ))}</div>
    </motion.div>
  );
}

const mockUsers = [
  { id: '1', name: 'Max Mustermann', email: 'max@firma.de', role: 'admin' as const, vehicles: 2, status: 'active' as const },
  { id: '2', name: 'Lisa König', email: 'lisa.koenig@firma.de', role: 'user' as const, vehicles: 1, status: 'active' as const },
  { id: '3', name: 'Thomas Braun', email: 'thomas.b@firma.de', role: 'user' as const, vehicles: 3, status: 'active' as const },
  { id: '4', name: 'Anna Schmidt', email: 'anna.schmidt@firma.de', role: 'user' as const, vehicles: 1, status: 'active' as const },
  { id: '5', name: 'Peter Wagner', email: 'p.wagner@firma.de', role: 'admin' as const, vehicles: 2, status: 'active' as const },
  { id: '6', name: 'Sarah Meyer', email: 's.meyer@firma.de', role: 'user' as const, vehicles: 0, status: 'blocked' as const },
];

function AdminUsers() {
  const { t } = useTranslation();
  const [search, setSearch] = useState('');
  const [roleFilter, setRoleFilter] = useState<'all' | 'admin' | 'user'>('all');
  const filtered = mockUsers.filter(u => {
    if (search && !u.name.toLowerCase().includes(search.toLowerCase()) && !u.email.toLowerCase().includes(search.toLowerCase())) return false;
    if (roleFilter !== 'all' && u.role !== roleFilter) return false;
    return true;
  });

  return (
    <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white">{t('admin.users.title')}</h2>
        <button className="btn btn-primary"><Plus weight="bold" className="w-4 h-4" />{t('admin.users.addUser')}</button>
      </div>
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="relative flex-1"><MagnifyingGlass weight="regular" className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" /><input type="text" value={search} onChange={(e) => setSearch(e.target.value)} placeholder={t('admin.users.searchPlaceholder')} className="input pl-11" /></div>
        <select value={roleFilter} onChange={(e) => setRoleFilter(e.target.value as any)} className="input w-auto"><option value="all">{t('admin.users.allRoles')}</option><option value="admin">Admin</option><option value="user">User</option></select>
      </div>
      <div className="card overflow-hidden"><div className="overflow-x-auto"><table className="w-full"><thead><tr className="bg-gray-50 dark:bg-gray-800/50">
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.users.name')}</th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.users.email')}</th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.users.role')}</th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.users.vehiclesCol')}</th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.users.status')}</th>
        <th className="text-right px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.users.actions')}</th>
      </tr></thead><tbody className="divide-y divide-gray-100 dark:divide-gray-800">
        {filtered.map((user, i) => (
          <motion.tr key={user.id} initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: i * 0.05 }} className="hover:bg-gray-50 dark:hover:bg-gray-800/30 transition-colors">
            <td className="px-6 py-4"><div className="flex items-center gap-3"><div className="avatar text-sm">{user.name.charAt(0)}</div><span className="font-medium text-gray-900 dark:text-white">{user.name}</span></div></td>
            <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">{user.email}</td>
            <td className="px-6 py-4"><span className={`badge ${user.role === 'admin' ? 'badge-error' : 'badge-info'}`}>{user.role === 'admin' ? 'Admin' : 'User'}</span></td>
            <td className="px-6 py-4 text-sm text-gray-700 dark:text-gray-300">{user.vehicles}</td>
            <td className="px-6 py-4"><span className={`badge ${user.status === 'active' ? 'badge-success' : 'badge-error'}`}>{user.status === 'active' ? t('common.active') : t('common.blocked')}</span></td>
            <td className="px-6 py-4 text-right"><div className="flex items-center justify-end gap-1"><button className="btn btn-ghost btn-icon btn-sm"><PencilSimple weight="regular" className="w-4 h-4" /></button><button className="btn btn-ghost btn-icon btn-sm text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20"><Trash weight="regular" className="w-4 h-4" /></button></div></td>
          </motion.tr>
        ))}
      </tbody></table></div>
      {filtered.length === 0 && <div className="p-12 text-center"><Users weight="light" className="w-16 h-16 text-gray-300 dark:text-gray-700 mx-auto mb-4" /><p className="text-gray-500 dark:text-gray-400">{t('admin.users.noUsers')}</p></div>}
      </div>
    </motion.div>
  );
}

const mockBookings = [
  { id: 'b1', user: 'Max Mustermann', lot: 'Firmenparkplatz', slot: '47', type: 'Einmalig', period: 'Heute, 08:00–17:00', status: 'active' as const },
  { id: 'b2', user: 'Lisa König', lot: 'Tiefgarage Nord', slot: '12', type: 'Dauer', period: '01.02. – 28.02.2026', status: 'active' as const },
  { id: 'b3', user: 'Thomas Braun', lot: 'Firmenparkplatz', slot: '51', type: 'Mehrtägig', period: '05.02. – 07.02.2026', status: 'completed' as const },
  { id: 'b4', user: 'Anna Schmidt', lot: 'Tiefgarage Nord', slot: '3', type: 'Einmalig', period: 'Heute, 09:00–14:00', status: 'active' as const },
  { id: 'b5', user: 'Peter Wagner', lot: 'Firmenparkplatz', slot: '22', type: 'Einmalig', period: 'Gestern, 07:30–18:00', status: 'completed' as const },
  { id: 'b6', user: 'Sarah Meyer', lot: 'Firmenparkplatz', slot: '35', type: 'Einmalig', period: '03.02.2026, 10:00–15:00', status: 'cancelled' as const },
];

function AdminBookings() {
  const { t } = useTranslation();
  const [statusFilter, setStatusFilter] = useState<'all' | 'active' | 'completed' | 'cancelled'>('all');
  const [lotFilter, setLotFilter] = useState('all');
  const [selected, setSelected] = useState<Set<string>>(new Set());

  const bookingStatusConfig: Record<string, { label: string; class: string }> = {
    active: { label: t('bookings.statusActive'), class: 'badge-success' },
    completed: { label: t('bookings.statusCompleted'), class: 'badge-gray' },
    cancelled: { label: t('bookings.statusCancelled'), class: 'badge-error' },
  };

  const filtered = mockBookings.filter(b => { if (statusFilter !== 'all' && b.status !== statusFilter) return false; if (lotFilter !== 'all' && b.lot !== lotFilter) return false; return true; });
  function toggleSelect(id: string) { setSelected(prev => { const next = new Set(prev); next.has(id) ? next.delete(id) : next.add(id); return next; }); }
  function toggleAll() { selected.size === filtered.length ? setSelected(new Set()) : setSelected(new Set(filtered.map(b => b.id))); }

  return (
    <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white">{t('admin.bookings.title')}</h2>
        {selected.size > 0 && <div className="flex items-center gap-2"><span className="text-sm text-gray-500">{t('admin.bookings.selected', { count: selected.size })}</span><button className="btn btn-sm btn-danger"><XCircle weight="bold" className="w-4 h-4" />{t('bookings.cancelBtn')}</button></div>}
      </div>
      <div className="flex flex-col sm:flex-row gap-3">
        <select value={lotFilter} onChange={(e) => setLotFilter(e.target.value)} className="input w-auto"><option value="all">{t('admin.bookings.allLots')}</option><option value="Firmenparkplatz">Firmenparkplatz</option><option value="Tiefgarage Nord">Tiefgarage Nord</option></select>
        <select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value as any)} className="input w-auto"><option value="all">{t('admin.bookings.allStatus')}</option><option value="active">{t('bookings.statusActive')}</option><option value="completed">{t('bookings.statusCompleted')}</option><option value="cancelled">{t('bookings.statusCancelled')}</option></select>
      </div>
      <div className="card overflow-hidden"><div className="overflow-x-auto"><table className="w-full"><thead><tr className="bg-gray-50 dark:bg-gray-800/50">
        <th className="px-6 py-3 text-left"><input type="checkbox" checked={selected.size === filtered.length && filtered.length > 0} onChange={toggleAll} className="w-4 h-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500" /></th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.bookings.user')}</th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.bookings.lot')}</th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.bookings.slot')}</th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.bookings.type')}</th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.bookings.period')}</th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.bookings.status')}</th>
        <th className="text-right px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.users.actions')}</th>
      </tr></thead><tbody className="divide-y divide-gray-100 dark:divide-gray-800">
        {filtered.map((b, i) => { const sc = bookingStatusConfig[b.status]; return (
          <motion.tr key={b.id} initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: i * 0.04 }} className={`transition-colors ${selected.has(b.id) ? 'bg-primary-50 dark:bg-primary-900/10' : 'hover:bg-gray-50 dark:hover:bg-gray-800/30'}`}>
            <td className="px-6 py-4"><input type="checkbox" checked={selected.has(b.id)} onChange={() => toggleSelect(b.id)} className="w-4 h-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500" /></td>
            <td className="px-6 py-4 font-medium text-gray-900 dark:text-white">{b.user}</td>
            <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">{b.lot}</td>
            <td className="px-6 py-4"><span className="font-mono font-bold text-gray-900 dark:text-white">{b.slot}</span></td>
            <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">{b.type}</td>
            <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">{b.period}</td>
            <td className="px-6 py-4"><span className={`badge ${sc.class}`}>{sc.label}</span></td>
            <td className="px-6 py-4 text-right">{b.status === 'active' && <button className="btn btn-ghost btn-sm text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20"><XCircle weight="bold" className="w-4 h-4" />{t('bookings.cancelBtn')}</button>}</td>
          </motion.tr>
        ); })}
      </tbody></table></div>
      {filtered.length === 0 && <div className="p-12 text-center"><ListChecks weight="light" className="w-16 h-16 text-gray-300 dark:text-gray-700 mx-auto mb-4" /><p className="text-gray-500 dark:text-gray-400">{t('admin.bookings.noBookings')}</p></div>}
      </div>
    </motion.div>
  );
}

export function AdminPage() {
  const { t } = useTranslation();
  return (
    <div>
      <div className="mb-2"><h1 className="text-2xl font-bold text-gray-900 dark:text-white">{t('admin.title')}</h1><p className="text-gray-500 dark:text-gray-400 mt-1">{t('admin.subtitle')}</p></div>
      <AdminNav />
      <Routes><Route path="/" element={<AdminOverview />} /><Route path="/lots" element={<AdminLots />} /><Route path="/users" element={<AdminUsers />} /><Route path="/bookings" element={<AdminBookings />} /><Route path="/branding" element={<AdminBrandingPage />} /></Routes>
    </div>
  );
}
