import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';
import {
  CalendarBlank,
  Clock,
  Car,
  X,
  SpinnerGap,
  CheckCircle,
  XCircle,
  ArrowClockwise,
  Warning,
  MapPin,
  CalendarPlus,
  Repeat,
  PencilSimple,
  Timer,
  CalendarCheck,
} from '@phosphor-icons/react';
import { api, Booking, Vehicle } from '../api/client';
import toast from 'react-hot-toast';
import { format, formatDistanceToNow, isPast, isFuture } from 'date-fns';
import { de } from 'date-fns/locale';
import { ConfirmDialog } from '../components/ConfirmDialog';

const bookingTypeConfig = {
  einmalig: { label: 'Einmalig', class: 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400' },
  mehrtaegig: { label: 'Mehrtägig', class: 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400' },
  dauer: { label: 'Dauer', class: 'bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400' },
};

const statusConfig = {
  active: { label: 'Aktiv', class: 'badge-success', icon: Clock },
  completed: { label: 'Abgeschlossen', class: 'badge-gray', icon: CheckCircle },
  cancelled: { label: 'Storniert', class: 'badge-error', icon: XCircle },
};

function BookingCard({ booking, onCancel, cancelling, vehiclePhoto }: { booking: Booking; onCancel: (id: string) => void; cancelling: string | null; vehiclePhoto?: string }) {
  const isExpiringSoon = booking.status === 'active' && new Date(booking.end_time).getTime() - Date.now() < 30 * 60 * 1000 && !isFuture(new Date(booking.start_time));
  const isUpcoming = booking.status === 'active' && isFuture(new Date(booking.start_time));
  const isActive = booking.status === 'active' && !isUpcoming;
  const isPastBooking = booking.status === 'completed' || booking.status === 'cancelled';
  const cfg = statusConfig[booking.status];
  const StatusIcon = cfg.icon;
  const bType = booking.booking_type || 'einmalig';
  const typeConf = bookingTypeConfig[bType];

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, x: -100 }}
      className={`card p-6 shadow-md dark:shadow-gray-900/50 border-l-4 transition-all hover:shadow-lg hover:-translate-y-0.5 ${
        isPastBooking ? 'border-l-gray-300 dark:border-l-gray-600 opacity-80' :
        isExpiringSoon ? 'border-l-amber-500' :
        isUpcoming ? 'border-l-blue-500' :
        'border-l-primary-500'
      }`}
    >
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className={`w-14 h-14 rounded-2xl flex items-center justify-center ${
            isPastBooking ? 'bg-gray-100 dark:bg-gray-800' :
            isExpiringSoon ? 'bg-amber-100 dark:bg-amber-900/30' :
            'bg-primary-100 dark:bg-primary-900/30'
          }`}>
            <span className={`text-xl font-bold ${
              isPastBooking ? 'text-gray-500 dark:text-gray-400' :
              isExpiringSoon ? 'text-amber-600 dark:text-amber-400' :
              'text-primary-600 dark:text-primary-400'
            }`}>
              {booking.slot_number}
            </span>
          </div>
          <div>
            <p className="font-semibold text-gray-900 dark:text-white">{booking.lot_name}</p>
            <div className="flex items-center gap-1.5 text-sm text-gray-500 dark:text-gray-400">
              <MapPin weight="regular" className="w-3.5 h-3.5" />
              Stellplatz {booking.slot_number}
            </div>
          </div>
        </div>
        <div className="flex flex-col items-end gap-2">
          <span className={`badge ${cfg.class}`}>
            <StatusIcon weight="fill" className="w-3 h-3" />
            {cfg.label}
          </span>
          <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[11px] font-semibold ${typeConf.class}`}>
            {bType === 'dauer' && <Repeat weight="bold" className="w-3 h-3" />}
            {bType === 'mehrtaegig' && <CalendarCheck weight="bold" className="w-3 h-3" />}
            {bType === 'einmalig' && <Clock weight="bold" className="w-3 h-3" />}
            {typeConf.label}
            {bType === 'dauer' && booking.dauer_interval && (
              <span className="ml-0.5">· {booking.dauer_interval === 'weekly' ? 'Wöchentlich' : 'Monatlich'}</span>
            )}
          </span>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-3 text-sm mb-4">
        <div className="flex items-center gap-2 text-gray-600 dark:text-gray-400">
          {vehiclePhoto ? (
            <img src={vehiclePhoto} alt="" className="w-8 h-8 rounded-full object-cover flex-shrink-0" />
          ) : (
            <Car weight="regular" className="w-4 h-4" />
          )}
          <span className="font-mono">{booking.vehicle_plate || '—'}</span>
        </div>
        <div className="flex items-center gap-2 text-gray-600 dark:text-gray-400">
          <Timer weight="regular" className="w-4 h-4" />
          {bType === 'einmalig' ? (
            <span>{format(new Date(booking.start_time), 'HH:mm')} — {format(new Date(booking.end_time), 'HH:mm')} Uhr</span>
          ) : (
            <span>{format(new Date(booking.start_time), 'd. MMM', { locale: de })} — {format(new Date(booking.end_time), 'd. MMM yyyy', { locale: de })}</span>
          )}
        </div>
      </div>

      <div className="flex items-center justify-between pt-3 border-t border-gray-100 dark:border-gray-800">
        <p className={`text-sm ${isExpiringSoon ? 'text-amber-600 dark:text-amber-400 font-medium' : 'text-gray-500 dark:text-gray-400'}`}>
          {isExpiringSoon && <Warning weight="fill" className="w-3.5 h-3.5 inline mr-1" />}
          {isUpcoming ? (
            <>Beginnt {formatDistanceToNow(new Date(booking.start_time), { addSuffix: true, locale: de })}</>
          ) : isPastBooking ? (
            <>{format(new Date(booking.start_time), 'd. MMMM yyyy', { locale: de })}</>
          ) : (
            <>Endet {formatDistanceToNow(new Date(booking.end_time), { addSuffix: true, locale: de })}</>
          )}
        </p>
        <div className="flex items-center gap-2">
          {isActive && (
            <button className="btn btn-sm btn-secondary">
              <PencilSimple weight="bold" className="w-3.5 h-3.5" />
              Verlängern
            </button>
          )}
          {booking.status === 'active' && (
            <button
              onClick={() => onCancel(booking.id)}
              disabled={cancelling === booking.id}
              className="btn btn-sm btn-ghost text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20"
            >
              {cancelling === booking.id ? (
                <SpinnerGap weight="bold" className="w-4 h-4 animate-spin" />
              ) : (
                <>
                  <X weight="bold" className="w-4 h-4" />
                  Stornieren
                </>
              )}
            </button>
          )}
        </div>
      </div>
    </motion.div>
  );
}

function SectionHeader({ icon: Icon, title, count, color }: { icon: any; title: string; count: number; color: string }) {
  return (
    <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
      <Icon weight="fill" className={`w-5 h-5 ${color}`} />
      {title}
      <span className="badge badge-gray text-xs">{count}</span>
    </h2>
  );
}

function EmptySection({ icon: Icon, text, showAction = false }: { icon: any; text: string; showAction?: boolean }) {
  return (
    <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="card p-12 text-center">
      <Icon weight="light" className="w-20 h-20 text-gray-200 dark:text-gray-700 mx-auto mb-4" />
      <p className="text-gray-500 dark:text-gray-400 mb-4">{text}</p>
      {showAction && (
        <Link to="/book" className="btn btn-primary">
          <CalendarPlus weight="bold" className="w-4 h-4" />
          Jetzt buchen
        </Link>
      )}
    </motion.div>
  );
}

export function BookingsPage() {
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [vehicles, setVehicles] = useState<Vehicle[]>([]);
  const [loading, setLoading] = useState(true);
  const [cancelling, setCancelling] = useState<string | null>(null);
  const [confirmCancelId, setConfirmCancelId] = useState<string | null>(null);

  useEffect(() => { loadData(); }, []);

  async function loadData() {
    try {
      const [bRes, vRes] = await Promise.all([api.getBookings(), api.getVehicles()]);
      if (bRes.success && bRes.data) setBookings(bRes.data);
      if (vRes.success && vRes.data) setVehicles(vRes.data);
    } finally { setLoading(false); }
  }

  async function loadBookings() { loadData(); }

  function getVehiclePhoto(plate?: string) {
    if (!plate) return undefined;
    return vehicles.find(v => v.license_plate === plate)?.photoUrl;
  }

  async function handleCancel(id: string) {
    setCancelling(id);
    const res = await api.cancelBooking(id);
    if (res.success) {
      setBookings(bookings.map(b => b.id === id ? { ...b, status: 'cancelled' as const } : b));
      toast.success('Buchung storniert');
    } else { toast.error('Stornierung fehlgeschlagen'); }
    setCancelling(null);
  }

  const now = new Date();
  const activeBookings = bookings.filter(b => b.status === 'active' && !isFuture(new Date(b.start_time)));
  const upcomingBookings = bookings.filter(b => b.status === 'active' && isFuture(new Date(b.start_time)));
  const pastBookings = bookings.filter(b => b.status === 'completed' || b.status === 'cancelled');

  function requestCancel(id: string) {
    setConfirmCancelId(id);
  }

  function confirmCancel() {
    if (confirmCancelId) {
      handleCancel(confirmCancelId);
      setConfirmCancelId(null);
    }
  }

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="h-8 w-64 skeleton" />
        {[1, 2, 3].map(i => <div key={i} className="h-40 skeleton rounded-2xl" />)}
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Meine Buchungen</h1>
          <p className="text-gray-500 dark:text-gray-400 mt-1">Übersicht Ihrer Parkplatz-Buchungen</p>
        </div>
        <button onClick={loadBookings} className="btn btn-secondary">
          <ArrowClockwise weight="bold" className="w-4 h-4" />
          Aktualisieren
        </button>
      </div>

      {/* Active Bookings */}
      <div>
        <SectionHeader icon={Clock} title="Aktive Buchungen" count={activeBookings.length} color="text-emerald-600" />
        {activeBookings.length === 0 ? (
          <EmptySection icon={CalendarBlank} text="Keine aktiven Buchungen" showAction />
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            <AnimatePresence>
              {activeBookings.map((bk) => (
                <BookingCard key={bk.id} booking={bk} onCancel={requestCancel} cancelling={cancelling} vehiclePhoto={getVehiclePhoto(bk.vehicle_plate)} />
              ))}
            </AnimatePresence>
          </div>
        )}
      </div>

      {/* Upcoming Bookings */}
      <div>
        <SectionHeader icon={CalendarPlus} title="Anstehende Buchungen" count={upcomingBookings.length} color="text-blue-600" />
        {upcomingBookings.length === 0 ? (
          <EmptySection icon={CalendarCheck} text="Keine anstehenden Buchungen" />
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            <AnimatePresence>
              {upcomingBookings.map((bk) => (
                <BookingCard key={bk.id} booking={bk} onCancel={requestCancel} cancelling={cancelling} vehiclePhoto={getVehiclePhoto(bk.vehicle_plate)} />
              ))}
            </AnimatePresence>
          </div>
        )}
      </div>

      {/* Past Bookings */}
      <div>
        <SectionHeader icon={CalendarBlank} title="Vergangene Buchungen" count={pastBookings.length} color="text-gray-400" />
        {pastBookings.length === 0 ? (
          <EmptySection icon={CheckCircle} text="Noch keine vergangenen Buchungen" />
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            {pastBookings.map((bk) => (
              <BookingCard key={bk.id} booking={bk} onCancel={requestCancel} cancelling={cancelling} vehiclePhoto={getVehiclePhoto(bk.vehicle_plate)} />
            ))}
          </div>
        )}
      </div>

      <ConfirmDialog
        open={!!confirmCancelId}
        title="Buchung wirklich stornieren?"
        message="Diese Aktion kann nicht rückgängig gemacht werden. Der Stellplatz wird für andere Benutzer freigegeben."
        confirmLabel="Stornieren"
        variant="danger"
        onConfirm={confirmCancel}
        onCancel={() => setConfirmCancelId(null)}
      />
    </div>
  );
}
