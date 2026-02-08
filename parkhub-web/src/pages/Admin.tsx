import { useEffect, useState } from 'react';
import { Routes, Route, Link, useLocation } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';
import {
  ChartBar, Buildings, Users, ListChecks, Plus, CheckCircle, TrendUp, CaretRight,
  SpinnerGap, MagnifyingGlass, XCircle, Trash, PencilSimple,
  Lightning, Pulse, ShieldCheck, Clock, House, Prohibit, Palette, GearSix, ArrowsClockwise,
} from '@phosphor-icons/react';
import { api, ParkingLot, ParkingLotDetailed, User, Booking, AdminStats } from '../api/client';
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
    { name: t('admin.tabs.system', 'System'), path: '/admin/system', icon: GearSix },
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
  const [stats, setStats] = useState<AdminStats | null>(null);
  const [showReset, setShowReset] = useState(false);
  const [resetInput, setResetInput] = useState('');
  const [resetting, setResetting] = useState(false);

  async function handleReset() {
    if (resetInput !== 'RESET') return;
    setResetting(true);
    try {
      const token = localStorage.getItem('parkhub_token');
      const res = await fetch('/api/v1/admin/reset', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token },
        body: JSON.stringify({ confirm: 'RESET' }),
      });
      const data = await res.json();
      if (data.success) {
        localStorage.removeItem('parkhub_token');
        window.location.href = '/login';
      }
    } finally { setResetting(false); }
  }

  useEffect(() => {
    (async () => {
      try {
        const res = await api.getAdminStats();
        if (res.success && res.data) setStats(res.data);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const statCards = [
    { label: t('admin.overview.totalSlots'), value: stats ? String(stats.total_lots) : '0', icon: Buildings, color: 'bg-blue-100 dark:bg-blue-900/30', iconColor: 'text-blue-600 dark:text-blue-400' },
    { label: t('admin.overview.activeBookings'), value: stats ? String(stats.active_bookings) : '0', icon: Clock, color: 'bg-emerald-100 dark:bg-emerald-900/30', iconColor: 'text-emerald-600 dark:text-emerald-400' },
    { label: t('admin.overview.occupancyToday'), value: '-', icon: TrendUp, color: 'bg-amber-100 dark:bg-amber-900/30', iconColor: 'text-amber-600 dark:text-amber-400' },
    { label: t('admin.overview.homeofficeToday'), value: '-', icon: House, color: 'bg-purple-100 dark:bg-purple-900/30', iconColor: 'text-purple-600 dark:text-purple-400' },
  ];

  const systemStatus = [{ name: t('admin.overview.backendApi'), status: 'online' }, { name: t('admin.overview.database'), status: 'online' }, { name: t('admin.overview.authService'), status: 'online' }];

  if (loading) return <div className="space-y-8"><div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">{[1,2,3,4].map(i => <div key={i} className="h-32 skeleton rounded-2xl" />)}</div><div className="h-64 skeleton rounded-2xl" /></div>;

  return (
    <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="space-y-8">
      <h2 className="text-xl font-semibold text-gray-900 dark:text-white">{t('admin.overview.title')}</h2>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {statCards.map((s, i) => { const Icon = s.icon; return (
          <motion.div key={s.label} initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: i * 0.08 }} className="stat-card">
            <div className="flex items-center justify-between"><div><p className="stat-label">{s.label}</p><p className="stat-value text-gray-900 dark:text-white">{s.value}</p></div><div className={`w-12 h-12 ${s.color} rounded-xl flex items-center justify-center`}><Icon weight="fill" className={`w-6 h-6 ${s.iconColor}`} /></div></div>
          </motion.div>
        ); })}
      </div>
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 card p-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2"><Pulse weight="fill" className="w-5 h-5 text-primary-600" />{t('admin.overview.recentActivity')}</h3>
          <div className="p-8 text-center text-gray-400 dark:text-gray-500">
            <Pulse weight="light" className="w-12 h-12 mx-auto mb-2 opacity-50" />
            <p className="text-sm">{t('admin.overview.noActivity', 'No recent activity')}</p>
          </div>
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
      {/* Database Reset - Danger Zone */}
      <div className="card p-6 border-2 border-red-200 dark:border-red-900/50">
        <h3 className="text-lg font-semibold text-red-600 dark:text-red-400 mb-2 flex items-center gap-2">
          <Trash weight="fill" className="w-5 h-5" />{t('admin.reset.title')}
        </h3>
        {!showReset ? (
          <button onClick={() => setShowReset(true)} className="btn bg-red-600 hover:bg-red-700 text-white">
            <Trash weight="bold" className="w-4 h-4" />{t('admin.reset.button')}
          </button>
        ) : (
          <div className="space-y-3">
            <p className="text-sm text-red-600 dark:text-red-400 font-medium">{t('admin.reset.warning')}</p>
            <div>
              <label className="label text-red-600">{t('admin.reset.inputLabel')}</label>
              <input type="text" value={resetInput} onChange={e => setResetInput(e.target.value)} placeholder={t('admin.reset.inputPlaceholder')} className="input border-red-300 dark:border-red-700" />
            </div>
            <div className="flex gap-2">
              <button onClick={handleReset} disabled={resetInput !== 'RESET' || resetting} className="btn bg-red-600 hover:bg-red-700 text-white disabled:opacity-50">
                <Trash weight="bold" className="w-4 h-4" />{resetting ? t('common.loading') : t('admin.reset.confirm')}
              </button>
              <button onClick={() => { setShowReset(false); setResetInput(''); }} className="btn btn-secondary">{t('common.cancel')}</button>
            </div>
          </div>
        )}
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
          <div className="card p-6"><h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">{t('admin.lots.createLot')}</h3><LotLayoutEditor onSave={async (layout, name) => {
                if (!name?.trim()) return;
                try {
                  const token = localStorage.getItem('parkhub_token');
                  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
                  if (token) headers['Authorization'] = `Bearer ${token}`;
                  const totalSlots = layout.rows.reduce((sum, r) => sum + r.slots.length, 0);
                  const createRes = await fetch('/api/v1/lots', {
                    method: 'POST',
                    headers,
                    body: JSON.stringify({ name, address: name, total_slots: totalSlots }),
                  });
                  const createData = await createRes.json();
                  if (createData.success && createData.data?.id && layout.rows.length > 0) {
                    await fetch(`/api/v1/lots/${createData.data.id}/layout`, {
                      method: 'PUT',
                      headers,
                      body: JSON.stringify(layout),
                    });
                  }
                  await loadLots();
                  setShowNewEditor(false);
                } catch { setShowNewEditor(false); }
              }} onCancel={() => setShowNewEditor(false)} /></div>
        </motion.div>
      )}</AnimatePresence>
      {lots.length === 0 && !showNewEditor && (
        <div className="p-12 text-center"><Buildings weight="light" className="w-16 h-16 text-gray-300 dark:text-gray-700 mx-auto mb-4" /><p className="text-gray-500 dark:text-gray-400">{t('admin.lots.noLots', 'No parking lots configured yet.')}</p></div>
      )}
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
              <div className="card p-6 mt-2 border-l-4 border-l-primary-500"><LotLayoutEditor initialLayout={editingLayout.layout} lotName={editingLayout.name} onSave={async (layout, _name) => {
                    try {
                      const token = localStorage.getItem('parkhub_token');
                      const headers: Record<string, string> = { 'Content-Type': 'application/json' };
                      if (token) headers['Authorization'] = `Bearer ${token}`;
                      if (layout.rows.length > 0) {
                        await fetch(`/api/v1/lots/${editingLotId}/layout`, {
                          method: 'PUT',
                          headers,
                          body: JSON.stringify(layout),
                        });
                      }
                      await loadLots();
                      setEditingLotId(null);
                    } catch { setEditingLotId(null); }
                  }} onCancel={() => setEditingLotId(null)} /></div>
            </motion.div>
          )}</AnimatePresence>
        </div>
      ))}</div>
    </motion.div>
  );
}

function AdminUsers() {
  const { t } = useTranslation();
  const [users, setUsers] = useState<(User & { vehicles?: number; status?: string })[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [roleFilter, setRoleFilter] = useState<'all' | 'admin' | 'user'>('all');

  useEffect(() => {
    (async () => {
      try {
        const res = await api.getAdminUsers();
        if (res.success && res.data) setUsers(res.data);
      } finally { setLoading(false); }
    })();
  }, []);

  const filtered = users.filter(u => {
    if (search && !u.name.toLowerCase().includes(search.toLowerCase()) && !u.email.toLowerCase().includes(search.toLowerCase())) return false;
    if (roleFilter !== 'all' && u.role !== roleFilter) return false;
    return true;
  });

  if (loading) return <div className="flex items-center justify-center h-64"><SpinnerGap weight="bold" className="w-8 h-8 text-primary-600 animate-spin" /></div>;

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
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.users.status')}</th>
        <th className="text-right px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.users.actions')}</th>
      </tr></thead><tbody className="divide-y divide-gray-100 dark:divide-gray-800">
        {filtered.map((user, i) => (
          <motion.tr key={user.id} initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: i * 0.05 }} className="hover:bg-gray-50 dark:hover:bg-gray-800/30 transition-colors">
            <td className="px-6 py-4"><div className="flex items-center gap-3"><div className="avatar text-sm">{user.name?.charAt(0) || '?'}</div><span className="font-medium text-gray-900 dark:text-white">{user.name}</span></div></td>
            <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">{user.email}</td>
            <td className="px-6 py-4"><span className={`badge ${user.role === 'admin' || user.role === 'superadmin' ? 'badge-error' : 'badge-info'}`}>{user.role === 'admin' || user.role === 'superadmin' ? 'Admin' : 'User'}</span></td>
            <td className="px-6 py-4"><span className="badge badge-success">{t('common.active')}</span></td>
            <td className="px-6 py-4 text-right"><div className="flex items-center justify-end gap-1"><button className="btn btn-ghost btn-icon btn-sm"><PencilSimple weight="regular" className="w-4 h-4" /></button><button className="btn btn-ghost btn-icon btn-sm text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20"><Trash weight="regular" className="w-4 h-4" /></button></div></td>
          </motion.tr>
        ))}
      </tbody></table></div>
      {filtered.length === 0 && <div className="p-12 text-center"><Users weight="light" className="w-16 h-16 text-gray-300 dark:text-gray-700 mx-auto mb-4" /><p className="text-gray-500 dark:text-gray-400">{t('admin.users.noUsers')}</p></div>}
      </div>
    </motion.div>
  );
}

function AdminBookings() {
  const { t } = useTranslation();
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [loading, setLoading] = useState(true);
  const [statusFilter, setStatusFilter] = useState<'all' | 'active' | 'completed' | 'cancelled'>('all');
  const [lotFilter, setLotFilter] = useState('all');
  const [selected, setSelected] = useState<Set<string>>(new Set());

  useEffect(() => {
    (async () => {
      try {
        const res = await api.getAdminBookings();
        if (res.success && res.data) setBookings(res.data);
      } finally { setLoading(false); }
    })();
  }, []);

  const bookingStatusConfig: Record<string, { label: string; class: string }> = {
    active: { label: t('bookings.statusActive'), class: 'badge-success' },
    completed: { label: t('bookings.statusCompleted'), class: 'badge-gray' },
    cancelled: { label: t('bookings.statusCancelled'), class: 'badge-error' },
  };

  const lotNames = [...new Set(bookings.map(b => b.lot_name))];
  const filtered = bookings.filter(b => { if (statusFilter !== 'all' && b.status !== statusFilter) return false; if (lotFilter !== 'all' && b.lot_name !== lotFilter) return false; return true; });
  function toggleSelect(id: string) { setSelected(prev => { const next = new Set(prev); next.has(id) ? next.delete(id) : next.add(id); return next; }); }
  function toggleAll() { selected.size === filtered.length ? setSelected(new Set()) : setSelected(new Set(filtered.map(b => b.id))); }

  if (loading) return <div className="flex items-center justify-center h-64"><SpinnerGap weight="bold" className="w-8 h-8 text-primary-600 animate-spin" /></div>;

  return (
    <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white">{t('admin.bookings.title')}</h2>
        {selected.size > 0 && <div className="flex items-center gap-2"><span className="text-sm text-gray-500">{t('admin.bookings.selected', { count: selected.size })}</span><button className="btn btn-sm btn-danger"><XCircle weight="bold" className="w-4 h-4" />{t('bookings.cancelBtn')}</button></div>}
      </div>
      <div className="flex flex-col sm:flex-row gap-3">
        <select value={lotFilter} onChange={(e) => setLotFilter(e.target.value)} className="input w-auto">
          <option value="all">{t('admin.bookings.allLots')}</option>
          {lotNames.map(name => <option key={name} value={name}>{name}</option>)}
        </select>
        <select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value as any)} className="input w-auto"><option value="all">{t('admin.bookings.allStatus')}</option><option value="active">{t('bookings.statusActive')}</option><option value="completed">{t('bookings.statusCompleted')}</option><option value="cancelled">{t('bookings.statusCancelled')}</option></select>
      </div>
      <div className="card overflow-hidden"><div className="overflow-x-auto"><table className="w-full"><thead><tr className="bg-gray-50 dark:bg-gray-800/50">
        <th className="px-6 py-3 text-left"><input type="checkbox" checked={selected.size === filtered.length && filtered.length > 0} onChange={toggleAll} className="w-4 h-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500" /></th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.bookings.lot')}</th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.bookings.slot')}</th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.bookings.type')}</th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.bookings.period')}</th>
        <th className="text-left px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.bookings.status')}</th>
        <th className="text-right px-6 py-3 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('admin.users.actions')}</th>
      </tr></thead><tbody className="divide-y divide-gray-100 dark:divide-gray-800">
        {filtered.map((b, i) => { const sc = bookingStatusConfig[b.status] || { label: b.status, class: 'badge-gray' }; return (
          <motion.tr key={b.id} initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: i * 0.04 }} className={`transition-colors ${selected.has(b.id) ? 'bg-primary-50 dark:bg-primary-900/10' : 'hover:bg-gray-50 dark:hover:bg-gray-800/30'}`}>
            <td className="px-6 py-4"><input type="checkbox" checked={selected.has(b.id)} onChange={() => toggleSelect(b.id)} className="w-4 h-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500" /></td>
            <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">{b.lot_name}</td>
            <td className="px-6 py-4"><span className="font-mono font-bold text-gray-900 dark:text-white">{b.slot_number}</span></td>
            <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">{b.booking_type || '-'}</td>
            <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">{new Date(b.start_time).toLocaleDateString()} – {new Date(b.end_time).toLocaleDateString()}</td>
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

export 
function AdminSystem() {
  const { t } = useTranslation();
  const [version, setVersion] = useState('');
  const [latestVersion, setLatestVersion] = useState<string | null>(null);
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [checking, setChecking] = useState(false);
  const [error, setError] = useState('');
  const [lastChecked, setLastChecked] = useState<string | null>(null);

  useEffect(() => {
    fetch('/api/v1/system/version')
      .then(r => r.json())
      .then(d => setVersion(d.version || ''))
      .catch(() => {});
  }, []);

  async function checkForUpdates() {
    setChecking(true);
    setError('');
    try {
      const token = localStorage.getItem('parkhub_token');
      const res = await fetch('/api/v1/admin/updates/check', {
        headers: { Authorization: 'Bearer ' + token },
      });
      const data = await res.json();
      setLatestVersion(data.latest || null);
      setUpdateAvailable(data.update_available || false);
      if (data.error) setError(data.error);
      setLastChecked(new Date().toLocaleString());
    } catch {
      setError(t('admin.version.error', 'Could not check for updates'));
    }
    setChecking(false);
  }

  return (
    <div className="space-y-6">
      <div className="card p-6">
        <div className="flex items-center gap-3 mb-4">
          <GearSix weight="fill" className="w-5 h-5 text-primary-600" />
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">{t('admin.version.title', 'System Information')}</h2>
        </div>
        <div className="space-y-4">
          <div className="flex items-center justify-between py-2 border-b border-gray-100 dark:border-gray-800">
            <span className="text-sm text-gray-600 dark:text-gray-400">{t('admin.version.current', 'Current Version')}</span>
            <span className="font-mono font-medium text-gray-900 dark:text-white">v{version || '...'}</span>
          </div>
          {latestVersion && (
            <div className="flex items-center justify-between py-2 border-b border-gray-100 dark:border-gray-800">
              <span className="text-sm text-gray-600 dark:text-gray-400">{t('admin.version.latest', 'Latest Version')}</span>
              <span className={'font-mono font-medium ' + (updateAvailable ? 'text-yellow-600 dark:text-yellow-400' : 'text-green-600 dark:text-green-400')}>
                v{latestVersion}
              </span>
            </div>
          )}
          {lastChecked && (
            <div className="flex items-center justify-between py-2 border-b border-gray-100 dark:border-gray-800">
              <span className="text-sm text-gray-600 dark:text-gray-400">{t('admin.version.lastChecked', 'Last checked')}</span>
              <span className="text-sm text-gray-500">{lastChecked}</span>
            </div>
          )}
          <div className="flex items-center justify-between py-2 border-b border-gray-100 dark:border-gray-800">
            <span className="text-sm text-gray-600 dark:text-gray-400">{t('admin.version.repoUrl', 'Repository')}</span>
            <a href="https://github.com/frostplexx/parkhub" target="_blank" rel="noopener noreferrer" className="text-sm text-primary-600 hover:underline">
              github.com/frostplexx/parkhub
            </a>
          </div>
        </div>
        <div className="mt-6 flex items-center gap-3">
          <button onClick={checkForUpdates} disabled={checking} className="btn-primary flex items-center gap-2 px-4 py-2 text-sm">
            <ArrowsClockwise weight="bold" className={'w-4 h-4' + (checking ? ' animate-spin' : '')} />
            {checking ? t('admin.version.checking', 'Checking...') : t('admin.version.checkForUpdates', 'Check for updates')}
          </button>
        </div>
        {error && <p className="mt-3 text-sm text-red-500">{error}</p>}
        {updateAvailable && (
          <div className="mt-4 p-4 rounded-lg bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800">
            <p className="text-sm font-medium text-yellow-800 dark:text-yellow-300">{t('admin.version.updateAvailable', 'Update available')}: v{latestVersion}</p>
            <p className="text-sm text-yellow-700 dark:text-yellow-400 mt-1">{t('admin.version.updateInstructions', 'To update, run the update script on the server or pull the latest version from the repository.')}</p>
          </div>
        )}
        {latestVersion && !updateAvailable && !error && (
          <div className="mt-4 p-4 rounded-lg bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800">
            <p className="text-sm font-medium text-green-800 dark:text-green-300">{t('admin.version.noUpdates', 'You are running the latest version.')}</p>
          </div>
        )}
      </div>
    </div>
  );
}

export function AdminPage() {
  const { t } = useTranslation();
  return (
    <div>
      <div className="mb-2"><h1 className="text-2xl font-bold text-gray-900 dark:text-white">{t('admin.title')}</h1><p className="text-gray-500 dark:text-gray-400 mt-1">{t('admin.subtitle')}</p></div>
      <AdminNav />
      <Routes><Route path="/" element={<AdminOverview />} /><Route path="/lots" element={<AdminLots />} /><Route path="/users" element={<AdminUsers />} /><Route path="/bookings" element={<AdminBookings />} /><Route path="/branding" element={<AdminBrandingPage />} /><Route path="/system" element={<AdminSystem />} /></Routes>
    </div>
  );
}
