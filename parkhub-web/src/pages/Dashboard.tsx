import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { motion } from 'framer-motion';
import {
  CalendarPlus,
  ChartLine,
  Clock,
  ArrowRight,
  Buildings,
  CheckCircle,
  Warning,
  TrendUp,
  CaretRight,
} from '@phosphor-icons/react';
import { House, Briefcase } from '@phosphor-icons/react';
import { api, ParkingLot, ParkingLotDetailed, Booking, HomeofficeSettings } from '../api/client';
import { useAuth } from '../context/AuthContext';
import { ParkingLotGrid } from '../components/ParkingLotGrid';
import { format, formatDistanceToNow } from 'date-fns';
import { de } from 'date-fns/locale';

export function DashboardPage() {
  const { user } = useAuth();
  const [lots, setLots] = useState<ParkingLot[]>([]);
  const [detailedLots, setDetailedLots] = useState<ParkingLotDetailed[]>([]);
  const [activeBookings, setActiveBookings] = useState<Booking[]>([]);
  const [hoSettings, setHoSettings] = useState<HomeofficeSettings | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, []);

  async function loadData() {
    try {
      const [lotsRes, bookingsRes, hoRes] = await Promise.all([
        api.getLots(),
        api.getBookings(),
        api.getHomeofficeSettings(),
      ]);
      if (hoRes.success && hoRes.data) setHoSettings(hoRes.data);

      if (lotsRes.success && lotsRes.data) {
        setLots(lotsRes.data);
        // Load detailed lot data for grid view
        const detailedPromises = lotsRes.data.map((lot) => api.getLotDetailed(lot.id));
        const detailedResults = await Promise.all(detailedPromises);
        setDetailedLots(
          detailedResults
            .filter((r) => r.success && r.data)
            .map((r) => r.data!)
        );
      }
      if (bookingsRes.success && bookingsRes.data) {
        setActiveBookings(bookingsRes.data.filter(b => b.status === 'active'));
      }
    } finally {
      setLoading(false);
    }
  }

  const todayDow = new Date().getDay() === 0 ? 6 : new Date().getDay() - 1;
  const todayStr = new Date().toISOString().slice(0, 10);
  const isHoToday = hoSettings ? (
    hoSettings.pattern.weekdays.includes(todayDow) ||
    hoSettings.singleDays.some(d => d.date === todayStr)
  ) : false;

  const totalSlots = lots.reduce((sum, lot) => sum + lot.total_slots, 0);
  const availableSlots = lots.reduce((sum, lot) => sum + lot.available_slots, 0);
  const occupancyRate = totalSlots > 0 ? Math.round((1 - availableSlots / totalSlots) * 100) : 0;

  const containerVariants = {
    hidden: { opacity: 0 },
    show: {
      opacity: 1,
      transition: { staggerChildren: 0.1 }
    }
  };

  const itemVariants = {
    hidden: { opacity: 0, y: 20 },
    show: { opacity: 1, y: 0 }
  };

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="h-8 w-64 skeleton" />
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {[1, 2, 3].map(i => (
            <div key={i} className="h-32 skeleton rounded-2xl" />
          ))}
        </div>
        <div className="h-64 skeleton rounded-2xl" />
      </div>
    );
  }

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="show"
      className="space-y-8"
    >
      {/* Welcome */}
      <motion.div variants={itemVariants}>
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
          Willkommen, {user?.name?.split(' ')[0]}
        </h1>
        <p className="text-gray-500 dark:text-gray-400 mt-1">
          {format(new Date(), "EEEE, d. MMMM yyyy", { locale: de })}
        </p>
      </motion.div>

      {/* Homeoffice Banner */}
      {isHoToday && hoSettings?.parkingSlot && (
        <motion.div variants={itemVariants} className="card bg-sky-50 dark:bg-sky-900/20 border border-sky-200 dark:border-sky-800 p-4">
          <div className="flex items-center gap-3">
            <House weight="fill" className="w-6 h-6 text-sky-600 dark:text-sky-400 flex-shrink-0" />
            <p className="font-medium text-sky-800 dark:text-sky-200">
              🏠 Heute Homeoffice — Ihr Stellplatz {hoSettings.parkingSlot.number} ist für Kollegen freigegeben
            </p>
          </div>
        </motion.div>
      )}
      {!isHoToday && (
        <motion.div variants={itemVariants}>
          <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-gray-100 dark:bg-gray-800 text-xs font-medium text-gray-600 dark:text-gray-400">
            <Briefcase weight="fill" className="w-3.5 h-3.5" />
            Im Büro
          </span>
        </motion.div>
      )}

      {/* Stats Grid */}
      <motion.div variants={itemVariants} className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* Available Slots */}
        <div className="stat-card">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm font-medium text-gray-500 dark:text-gray-400">
                Verfügbare Plätze
              </p>
              <div className="mt-2 flex items-baseline gap-2">
                <span className="stat-value text-emerald-600 dark:text-emerald-400">
                  {availableSlots}
                </span>
                <span className="text-gray-500 dark:text-gray-400">
                  / {totalSlots}
                </span>
              </div>
            </div>
            <div className="w-12 h-12 bg-emerald-100 dark:bg-emerald-900/30 rounded-xl flex items-center justify-center">
              <CheckCircle weight="fill" className="w-6 h-6 text-emerald-600 dark:text-emerald-400" />
            </div>
          </div>
          <div className="mt-4">
            <div className="progress">
              <div
                className="progress-bar bg-emerald-500"
                style={{ width: `${(availableSlots / totalSlots) * 100}%` }}
              />
            </div>
          </div>
        </div>

        {/* Occupancy Rate */}
        <div className="stat-card">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm font-medium text-gray-500 dark:text-gray-400">
                Auslastung
              </p>
              <div className="mt-2 flex items-baseline gap-2">
                <span className="stat-value text-gray-900 dark:text-white">
                  {occupancyRate}%
                </span>
                <span className="flex items-center gap-1 text-sm text-emerald-600">
                  <TrendUp weight="bold" className="w-4 h-4" />
                  Normal
                </span>
              </div>
            </div>
            <div className="w-12 h-12 bg-primary-100 dark:bg-primary-900/30 rounded-xl flex items-center justify-center">
              <ChartLine weight="fill" className="w-6 h-6 text-primary-600 dark:text-primary-400" />
            </div>
          </div>
          <div className="mt-4">
            <div className="progress">
              <div
                className={`progress-bar ${
                  occupancyRate > 80 ? 'bg-red-500' : occupancyRate > 60 ? 'bg-amber-500' : 'bg-primary-500'
                }`}
                style={{ width: `${occupancyRate}%` }}
              />
            </div>
          </div>
        </div>

        {/* Active Bookings */}
        <div className="stat-card">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm font-medium text-gray-500 dark:text-gray-400">
                Aktive Buchungen
              </p>
              <div className="mt-2">
                <span className="stat-value text-primary-600 dark:text-primary-400">
                  {activeBookings.length}
                </span>
              </div>
            </div>
            <div className="w-12 h-12 bg-amber-100 dark:bg-amber-900/30 rounded-xl flex items-center justify-center">
              <Clock weight="fill" className="w-6 h-6 text-amber-600 dark:text-amber-400" />
            </div>
          </div>
          <Link
            to="/bookings"
            className="mt-4 flex items-center gap-1 text-sm text-primary-600 dark:text-primary-400 font-medium hover:underline"
          >
            Alle anzeigen
            <CaretRight weight="bold" className="w-4 h-4" />
          </Link>
        </div>
      </motion.div>

      {/* Active Bookings */}
      {activeBookings.length > 0 && (
        <motion.div variants={itemVariants} className="card p-6">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
              Aktive Buchungen
            </h2>
            <Link to="/bookings" className="text-sm text-primary-600 dark:text-primary-400 font-medium hover:underline">
              Alle anzeigen
            </Link>
          </div>
          
          <div className="space-y-4">
            {activeBookings.slice(0, 3).map((booking, index) => (
              <motion.div
                key={booking.id}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: index * 0.1 }}
                className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800/50 rounded-xl"
              >
                <div className="flex items-center gap-4">
                  <div className="w-14 h-14 bg-primary-100 dark:bg-primary-900/30 rounded-xl flex items-center justify-center">
                    <span className="text-lg font-bold text-primary-600 dark:text-primary-400">
                      {booking.slot_number}
                    </span>
                  </div>
                  <div>
                    <p className="font-medium text-gray-900 dark:text-white">
                      {booking.lot_name}
                    </p>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      {booking.vehicle_plate || 'Kein Kennzeichen'}
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="font-medium text-gray-900 dark:text-white">
                    Bis {format(new Date(booking.end_time), 'HH:mm')} Uhr
                  </p>
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    {formatDistanceToNow(new Date(booking.end_time), { addSuffix: true, locale: de })}
                  </p>
                </div>
              </motion.div>
            ))}
          </div>
        </motion.div>
      )}

      {/* Schnellbuchung */}
      <motion.div variants={itemVariants} className="card p-6">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Schnellbuchung
        </h2>
        <p className="text-sm text-gray-500 dark:text-gray-400 mb-4">Häufig genutzte Stellplätze</p>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
          {[
            { slot: '47', lot: 'Firmenparkplatz', lotId: 'lot-1' },
            { slot: '2', lot: 'Tiefgarage Nord', lotId: 'lot-2' },
            { slot: '51', lot: 'Firmenparkplatz', lotId: 'lot-1' },
          ].map((item, i) => (
            <Link
              key={i}
              to={`/book?lot=${item.lotId}`}
              className="flex items-center gap-3 p-3 rounded-xl bg-gray-50 dark:bg-gray-800/50 hover:bg-primary-50 dark:hover:bg-primary-900/20 hover:shadow-md transition-all group"
            >
              <div className="w-12 h-12 bg-primary-100 dark:bg-primary-900/30 rounded-xl flex items-center justify-center group-hover:bg-primary-200 dark:group-hover:bg-primary-800/40 transition-colors">
                <span className="text-lg font-bold text-primary-600 dark:text-primary-400">{item.slot}</span>
              </div>
              <div>
                <p className="font-medium text-sm text-gray-900 dark:text-white">{item.lot}</p>
                <p className="text-xs text-gray-400">Stellplatz {item.slot}</p>
              </div>
            </Link>
          ))}
        </div>
      </motion.div>

      {/* Parking Lot Grid Overview */}
      {detailedLots.length > 0 && (
        <motion.div variants={itemVariants} className="space-y-4">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Parkplatz-Übersicht
          </h2>
          {detailedLots.filter((l) => l.layout).map((lot) => (
            <div key={lot.id} className="card p-6 shadow-md dark:shadow-gray-900/50">
              <h3 className="font-semibold text-gray-900 dark:text-white mb-4">
                {lot.name}
              </h3>
              <ParkingLotGrid layout={lot.layout!} interactive={false} />
            </div>
          ))}
        </motion.div>
      )}

      {/* Parking Lots */}
      <motion.div variants={itemVariants}>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Parkplätze
          </h2>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {lots.map((lot, index) => (
            <motion.div
              key={lot.id}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: index * 0.1 }}
            >
              <Link
                to={`/book?lot=${lot.id}`}
                className="card-hover block p-6"
              >
                <div className="flex items-start justify-between mb-4">
                  <div className="w-12 h-12 bg-gray-100 dark:bg-gray-800 rounded-xl flex items-center justify-center">
                    <Buildings weight="fill" className="w-6 h-6 text-gray-600 dark:text-gray-400" />
                  </div>
                  <div className={`badge ${
                    lot.available_slots === 0 ? 'badge-error' : lot.available_slots < 5 ? 'badge-warning' : 'badge-success'
                  }`}>
                    {lot.available_slots === 0 ? (
                      <>
                        <Warning weight="fill" className="w-3 h-3" />
                        Voll
                      </>
                    ) : (
                      <>
                        <CheckCircle weight="fill" className="w-3 h-3" />
                        {lot.available_slots} frei
                      </>
                    )}
                  </div>
                </div>
                
                <h3 className="font-semibold text-gray-900 dark:text-white mb-1">
                  {lot.name}
                </h3>
                <p className="text-sm text-gray-500 dark:text-gray-400 mb-4">
                  {lot.address}
                </p>
                
                <div className="flex items-center justify-between">
                  <div className="flex-1 mr-4">
                    <div className="progress h-1.5">
                      <div
                        className={`progress-bar ${
                          lot.available_slots === 0 ? 'bg-red-500' : 'bg-emerald-500'
                        }`}
                        style={{ width: `${(lot.available_slots / lot.total_slots) * 100}%` }}
                      />
                    </div>
                  </div>
                  <span className="text-sm text-gray-500 dark:text-gray-400">
                    {lot.total_slots - lot.available_slots}/{lot.total_slots}
                  </span>
                </div>
              </Link>
            </motion.div>
          ))}
        </div>
      </motion.div>

      {/* Quick Action */}
      <motion.div variants={itemVariants}>
        <div className="card bg-gradient-to-r from-primary-600 to-primary-700 p-6 text-white">
          <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
            <div className="flex items-center gap-4">
              <div className="w-14 h-14 bg-white/20 rounded-2xl flex items-center justify-center">
                <CalendarPlus weight="fill" className="w-7 h-7" />
              </div>
              <div>
                <h3 className="font-semibold text-lg">Jetzt Parkplatz buchen</h3>
                <p className="text-white/80 text-sm">
                  Wählen Sie einen freien Platz für heute oder die kommenden Tage
                </p>
              </div>
            </div>
            <Link
              to="/book"
              className="btn bg-white text-primary-700 hover:bg-white/90 focus:ring-white/50"
            >
              Jetzt buchen
              <ArrowRight weight="bold" className="w-5 h-5" />
            </Link>
          </div>
        </div>
      </motion.div>
    </motion.div>
  );
}
