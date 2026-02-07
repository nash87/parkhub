import { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Car, Plus, Trash, SpinnerGap, Star, X, CheckCircle, Camera, PencilSimple } from '@phosphor-icons/react';
import { api, Vehicle, generateCarPhotoSvg } from '../api/client';
import { useTranslation } from 'react-i18next';
import toast from 'react-hot-toast';
import { ConfirmDialog } from '../components/ConfirmDialog';

const colorMap: Record<string, string> = {
  'Schwarz': 'bg-gray-900', 'Weiß': 'bg-white border border-gray-300', 'Silber': 'bg-gray-400',
  'Grau': 'bg-gray-500', 'Blau': 'bg-blue-600', 'Rot': 'bg-red-600', 'Grün': 'bg-green-600', 'Gelb': 'bg-yellow-400',
};

function PhotoUpload({ photoUrl, color, onPhotoChange, t }: { photoUrl?: string; color?: string; onPhotoChange: (url: string) => void; t: any }) {
  const [dragOver, setDragOver] = useState(false);
  const placeholder = color ? generateCarPhotoSvg(color) : undefined;
  const displayUrl = photoUrl || placeholder;
  function handleFile(file: File) { if (!file.type.startsWith('image/')) return; onPhotoChange(URL.createObjectURL(file)); }

  return (
    <div className="flex flex-col items-center gap-3">
      <div onDragOver={(e) => { e.preventDefault(); setDragOver(true); }} onDragLeave={() => setDragOver(false)}
        onDrop={(e) => { e.preventDefault(); setDragOver(false); if (e.dataTransfer.files[0]) handleFile(e.dataTransfer.files[0]); }}
        onClick={() => { const input = document.createElement('input'); input.type = 'file'; input.accept = 'image/jpeg,image/png,image/webp'; input.onchange = () => { if (input.files?.[0]) handleFile(input.files[0]); }; input.click(); }}
        className={`relative w-[120px] h-[120px] rounded-2xl overflow-hidden cursor-pointer border-2 border-dashed transition-all flex items-center justify-center ${dragOver ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20 scale-105' : 'border-gray-300 dark:border-gray-600 hover:border-primary-400'}`}>
        {displayUrl ? <img src={displayUrl} alt="" className="w-full h-full object-cover" /> : (
          <div className="flex flex-col items-center gap-1 text-gray-400"><Camera weight="regular" className="w-8 h-8" /><span className="text-[10px] text-center leading-tight">{t('vehicles.uploadPhoto')}</span></div>
        )}
        {displayUrl && <div className="absolute inset-0 bg-black/0 hover:bg-black/40 transition-colors flex items-center justify-center group"><Camera weight="fill" className="w-6 h-6 text-white opacity-0 group-hover:opacity-100 transition-opacity" /></div>}
      </div>
      <span className="text-xs text-gray-400">{t('vehicles.uploadPhotoOrCamera')}</span>
    </div>
  );
}

function AddVehicleModal({ open, onClose, onSave }: { open: boolean; onClose: () => void; onSave: (v: Vehicle) => void }) {
  const { t } = useTranslation();
  const [formData, setFormData] = useState({ license_plate: '', make: '', model: '', color: '', is_default: false });
  const [photoUrl, setPhotoUrl] = useState<string | undefined>();
  const [saving, setSaving] = useState(false);

  const colorOptions = [
    { value: 'Schwarz', label: t('colors.black') }, { value: 'Weiß', label: t('colors.white') },
    { value: 'Silber', label: t('colors.silver') }, { value: 'Blau', label: t('colors.blue') },
    { value: 'Rot', label: t('colors.red') }, { value: 'Grün', label: t('colors.green') },
    { value: 'Grau', label: t('colors.gray') }, { value: 'Sonstige', label: t('colors.other') },
  ];

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!formData.license_plate.trim()) return;
    setSaving(true);
    const newVehicle: Vehicle = {
      id: 'v-' + Date.now(), user_id: 'demo-1', license_plate: formData.license_plate.toUpperCase(),
      make: formData.make || undefined, model: formData.model || undefined, color: formData.color || undefined,
      photoUrl: photoUrl || (formData.color ? generateCarPhotoSvg(formData.color) : undefined), is_default: formData.is_default,
    };
    await new Promise(r => setTimeout(r, 300));
    onSave(newVehicle);
    setFormData({ license_plate: '', make: '', model: '', color: '', is_default: false });
    setPhotoUrl(undefined);
    setSaving(false);
  }

  return (
    <AnimatePresence>
      {open && (
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="absolute inset-0 bg-black/50 backdrop-blur-sm" onClick={onClose} />
          <motion.div initial={{ opacity: 0, scale: 0.95, y: 20 }} animate={{ opacity: 1, scale: 1, y: 0 }} exit={{ opacity: 0, scale: 0.95, y: 20 }} transition={{ type: 'spring', damping: 25, stiffness: 300 }} className="relative w-full max-w-lg card p-0 shadow-2xl">
            <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100 dark:border-gray-800">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2"><Car weight="fill" className="w-5 h-5 text-primary-600" />{t('vehicles.newVehicle')}</h2>
              <button onClick={onClose} className="btn btn-ghost btn-icon"><X weight="bold" className="w-5 h-5" /></button>
            </div>
            <form onSubmit={handleSubmit} className="p-6 space-y-4">
              <PhotoUpload photoUrl={photoUrl} color={formData.color} onPhotoChange={setPhotoUrl} t={t} />
              <div><label className="label">{t('vehicles.plate')} *</label><input type="text" value={formData.license_plate} onChange={(e) => setFormData({ ...formData, license_plate: e.target.value.toUpperCase() })} placeholder={t('vehicles.platePlaceholder')} className="input font-mono text-lg tracking-wider" required autoFocus /></div>
              <div className="grid grid-cols-2 gap-4">
                <div><label className="label">{t('vehicles.make')}</label><input type="text" value={formData.make} onChange={(e) => setFormData({ ...formData, make: e.target.value })} placeholder={t('vehicles.makePlaceholder')} className="input" /></div>
                <div><label className="label">{t('vehicles.model')}</label><input type="text" value={formData.model} onChange={(e) => setFormData({ ...formData, model: e.target.value })} placeholder={t('vehicles.modelPlaceholder')} className="input" /></div>
              </div>
              <div><label className="label">{t('vehicles.color')}</label><select value={formData.color} onChange={(e) => setFormData({ ...formData, color: e.target.value })} className="input"><option value="">{t('vehicles.colorSelect')}</option>{colorOptions.map(c => <option key={c.value} value={c.value}>{c.label}</option>)}</select></div>
              <label className="flex items-center gap-3 p-3 rounded-xl bg-gray-50 dark:bg-gray-800/50 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">
                <input type="checkbox" checked={formData.is_default} onChange={(e) => setFormData({ ...formData, is_default: e.target.checked })} className="w-4 h-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500" />
                <div><span className="text-sm font-medium text-gray-900 dark:text-white">{t('vehicles.defaultVehicle')}</span><p className="text-xs text-gray-500 dark:text-gray-400">{t('vehicles.defaultVehicleDesc')}</p></div>
              </label>
              <div className="flex justify-end gap-3 pt-4 border-t border-gray-100 dark:border-gray-800">
                <button type="button" onClick={onClose} className="btn btn-secondary">{t('common.cancel')}</button>
                <button type="submit" disabled={saving || !formData.license_plate.trim()} className="btn btn-primary">
                  {saving ? <SpinnerGap weight="bold" className="w-5 h-5 animate-spin" /> : <><CheckCircle weight="bold" className="w-4 h-4" />{t('common.save')}</>}
                </button>
              </div>
            </form>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

export function VehiclesPage() {
  const { t } = useTranslation();
  const [vehicles, setVehicles] = useState<Vehicle[]>([]);
  const [loading, setLoading] = useState(true);
  const [showModal, setShowModal] = useState(false);
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);

  useEffect(() => { loadVehicles(); }, []);

  async function loadVehicles() { try { const res = await api.getVehicles(); if (res.success && res.data) setVehicles(res.data); } finally { setLoading(false); } }

  function handleAddVehicle(vehicle: Vehicle) {
    if (vehicle.is_default) setVehicles(prev => [...prev.map(v => ({ ...v, is_default: false })), vehicle]);
    else setVehicles(prev => [...prev, vehicle]);
    setShowModal(false);
    toast.success(t('vehicles.added'));
  }

  async function handleDelete(id: string) { setVehicles(vehicles.filter(v => v.id !== id)); toast.success(t('vehicles.removed')); }

  if (loading) return <div className="flex items-center justify-center h-64"><SpinnerGap weight="bold" className="w-8 h-8 text-primary-600 animate-spin" /></div>;

  return (
    <div className="space-y-8">
      <div className="flex items-center justify-between">
        <div><h1 className="text-2xl font-bold text-gray-900 dark:text-white">{t('vehicles.title')}</h1><p className="text-gray-500 dark:text-gray-400 mt-1">{t('vehicles.subtitle')}</p></div>
        <button onClick={() => setShowModal(true)} className="btn btn-primary"><Plus weight="bold" className="w-4 h-4" />{t('vehicles.add')}</button>
      </div>

      <AddVehicleModal open={showModal} onClose={() => setShowModal(false)} onSave={handleAddVehicle} />

      {vehicles.length === 0 ? (
        <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="card p-16 text-center">
          <Car weight="light" className="w-24 h-24 text-gray-200 dark:text-gray-700 mx-auto mb-4" />
          <p className="text-gray-500 dark:text-gray-400 mb-2 text-lg">{t('vehicles.noVehicles')}</p>
          <p className="text-sm text-gray-400 dark:text-gray-500 mb-6">{t('vehicles.noVehiclesDesc')}</p>
          <button onClick={() => setShowModal(true)} className="btn btn-primary"><Plus weight="bold" className="w-4 h-4" />{t('vehicles.addVehicle')}</button>
        </motion.div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {vehicles.map((vehicle, index) => {
            const colorClass = vehicle.color ? (colorMap[vehicle.color] || 'bg-gray-400') : 'bg-gray-400';
            return (
              <motion.div key={vehicle.id} initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: index * 0.05 }} className="card p-6 shadow-md dark:shadow-gray-900/50 hover:shadow-lg hover:-translate-y-0.5 transition-all">
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-4">
                    <div className="relative group">
                      {vehicle.photoUrl ? <img src={vehicle.photoUrl} alt={vehicle.license_plate} className="w-20 h-20 rounded-2xl object-cover" /> : (
                        <div className={`w-20 h-20 rounded-2xl flex items-center justify-center ${colorClass || 'bg-gray-100 dark:bg-gray-800'}`}><Car weight="fill" className="w-10 h-10 text-white/40" /></div>
                      )}
                      <button onClick={(e) => { e.stopPropagation(); const input = document.createElement('input'); input.type = 'file'; input.accept = 'image/*'; input.onchange = () => { if (input.files?.[0]) { const url = URL.createObjectURL(input.files[0]); setVehicles(prev => prev.map(v => v.id === vehicle.id ? { ...v, photoUrl: url } : v)); } }; input.click(); }}
                        className="absolute inset-0 rounded-2xl bg-black/0 group-hover:bg-black/40 transition-colors flex items-center justify-center">
                        <PencilSimple weight="bold" className="w-5 h-5 text-white opacity-0 group-hover:opacity-100 transition-opacity" />
                      </button>
                      <div className={`absolute -bottom-1 -right-1 w-5 h-5 rounded-full ${colorClass} ring-2 ring-white dark:ring-gray-900`} />
                    </div>
                    <div>
                      <p className="text-xl font-bold text-gray-900 dark:text-white font-mono tracking-wider">{vehicle.license_plate}</p>
                      {(vehicle.make || vehicle.model) && <p className="text-sm text-gray-600 dark:text-gray-400 font-medium">{vehicle.make} {vehicle.model}</p>}
                      {vehicle.color && <div className="flex items-center gap-1.5 mt-1"><div className={`w-2.5 h-2.5 rounded-full ${colorClass}`} /><span className="text-xs text-gray-500 dark:text-gray-500">{vehicle.color}</span></div>}
                    </div>
                  </div>
                  <button onClick={() => setConfirmDeleteId(vehicle.id)} className="btn btn-ghost btn-icon text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20"><Trash weight="regular" className="w-5 h-5" /></button>
                </div>
                {vehicle.is_default && <div className="mt-4 pt-4 border-t border-gray-100 dark:border-gray-800"><span className="badge badge-info"><Star weight="fill" className="w-3 h-3" />{t('vehicles.isDefault')}</span></div>}
              </motion.div>
            );
          })}
        </div>
      )}

      <ConfirmDialog open={!!confirmDeleteId} title={t('confirm.deleteVehicleTitle')} message={t('confirm.deleteVehicleMessage')} confirmLabel={t('confirm.deleteVehicleConfirm')} variant="danger"
        onConfirm={() => { if (confirmDeleteId) handleDelete(confirmDeleteId); setConfirmDeleteId(null); }} onCancel={() => setConfirmDeleteId(null)} />
    </div>
  );
}
