import { useEffect, useState } from 'react';
import { useSearchParams, useNavigate } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Car,
  Clock,
  CheckCircle,
  Warning,
  SpinnerGap,
  MapPin,
  CalendarBlank,
  Repeat,
} from '@phosphor-icons/react';
import { api, ParkingLot, ParkingLotDetailed, Vehicle, SlotConfig } from '../api/client';
import { ParkingLotGrid } from '../components/ParkingLotGrid';
import toast from 'react-hot-toast';
import { format, addMinutes, addDays, differenceInDays } from 'date-fns';
import { de } from 'date-fns/locale';

function BookingSuccessModal({ open, onDashboard, onNewBooking, summary }: {
  open: boolean;
  onDashboard: () => void;
  onNewBooking: () => void;
  summary: { lot: string; slot: string; type: string; time: string; plate: string };
}) {
  return (
    <AnimatePresence>
      {open && (
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <motion.div className="absolute inset-0 bg-black/50 backdrop-blur-sm" />
          <motion.div
            initial={{ opacity: 0, scale: 0.8 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.8 }}
            transition={{ type: 'spring', damping: 20, stiffness: 300 }}
            className="relative w-full max-w-md card p-8 shadow-2xl text-center"
          >
            {/* Animated Checkmark */}
            <motion.div
              initial={{ scale: 0 }}
              animate={{ scale: 1 }}
              transition={{ delay: 0.2, type: 'spring', damping: 12 }}
              className="w-20 h-20 bg-emerald-100 dark:bg-emerald-900/30 rounded-full flex items-center justify-center mx-auto mb-6"
            >
              <motion.div
                initial={{ scale: 0, rotate: -180 }}
                animate={{ scale: 1, rotate: 0 }}
                transition={{ delay: 0.4, type: 'spring', damping: 15 }}
              >
                <CheckCircle weight="fill" className="w-12 h-12 text-emerald-500" />
              </motion.div>
            </motion.div>

            <motion.h2
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.5 }}
              className="text-2xl font-bold text-gray-900 dark:text-white mb-2"
            >
              Buchung erfolgreich!
            </motion.h2>
            <motion.p
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.6 }}
              className="text-gray-500 dark:text-gray-400 mb-6"
            >
              Ihr Parkplatz wurde reserviert
            </motion.p>

            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.7 }}
              className="bg-gray-50 dark:bg-gray-800/50 rounded-xl p-4 mb-6 text-left space-y-2"
            >
              <div className="flex justify-between text-sm">
                <span className="text-gray-500 dark:text-gray-400">Parkplatz</span>
                <span className="font-medium text-gray-900 dark:text-white">{summary.lot}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-500 dark:text-gray-400">Stellplatz</span>
                <span className="font-bold text-primary-600 dark:text-primary-400">{summary.slot}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-500 dark:text-gray-400">Typ</span>
                <span className="font-medium text-gray-900 dark:text-white">{summary.type}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-500 dark:text-gray-400">Zeit</span>
                <span className="font-medium text-gray-900 dark:text-white">{summary.time}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-500 dark:text-gray-400">Kennzeichen</span>
                <span className="font-mono font-medium text-gray-900 dark:text-white">{summary.plate}</span>
              </div>
            </motion.div>

            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.8 }}
              className="flex gap-3"
            >
              <button onClick={onNewBooking} className="btn btn-secondary flex-1">
                Weitere Buchung
              </button>
              <button onClick={onDashboard} className="btn btn-primary flex-1">
                Zum Dashboard
              </button>
            </motion.div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

const DURATION_OPTIONS = [
  { value: 30, label: '30 Min' },
  { value: 60, label: '1 Std' },
  { value: 120, label: '2 Std' },
  { value: 240, label: '4 Std' },
  { value: 480, label: '8 Std' },
  { value: 720, label: '12 Std' },
];

type BookingType = 'einmalig' | 'mehrtaegig' | 'dauer';
type DauerInterval = 'weekly' | 'monthly';

export function BookPage() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const preselectedLot = searchParams.get('lot');

  const [lots, setLots] = useState<ParkingLot[]>([]);
  const [selectedLot, setSelectedLot] = useState<string>(preselectedLot || '');
  const [detailedLot, setDetailedLot] = useState<ParkingLotDetailed | null>(null);
  const [selectedSlot, setSelectedSlot] = useState<SlotConfig | null>(null);
  const [vehicles, setVehicles] = useState<Vehicle[]>([]);
  const [selectedVehicle, setSelectedVehicle] = useState<string>('');
  const [customPlate, setCustomPlate] = useState('');
  const [duration, setDuration] = useState(60);
  const [loading, setLoading] = useState(true);
  const [booking, setBooking] = useState(false);
  const [showSuccess, setShowSuccess] = useState(false);
  const [successSummary, setSuccessSummary] = useState({ lot: '', slot: '', type: '', time: '', plate: '' });

  // Enhanced booking state
  const [bookingType, setBookingType] = useState<BookingType>('einmalig');
  const [startDate, setStartDate] = useState(format(new Date(), 'yyyy-MM-dd'));
  const [endDate, setEndDate] = useState(format(addDays(new Date(), 3), 'yyyy-MM-dd'));
  const [dauerInterval, setDauerInterval] = useState<DauerInterval>('monthly');
  const [dauerDays, setDauerDays] = useState<number[]>([1, 3]); // Mon, Wed

  useEffect(() => {
    loadInitialData();
  }, []);

  useEffect(() => {
    if (selectedLot) {
      loadDetailedLot(selectedLot);
    }
  }, [selectedLot]);

  async function loadInitialData() {
    try {
      const [lotsRes, vehiclesRes] = await Promise.all([
        api.getLots(),
        api.getVehicles(),
      ]);
      if (lotsRes.success && lotsRes.data) {
        setLots(lotsRes.data);
        if (preselectedLot) setSelectedLot(preselectedLot);
      }
      if (vehiclesRes.success && vehiclesRes.data) {
        setVehicles(vehiclesRes.data);
        const defaultVehicle = vehiclesRes.data.find(v => v.is_default);
        if (defaultVehicle) setSelectedVehicle(defaultVehicle.id);
      }
    } finally {
      setLoading(false);
    }
  }

  async function loadDetailedLot(lotId: string) {
    const res = await api.getLotDetailed(lotId);
    if (res.success && res.data) {
      setDetailedLot(res.data);
    }
  }

  function handleSlotSelect(slot: SlotConfig) {
    if (slot.status === 'available') {
      setSelectedSlot(slot);
    }
  }

  async function handleBook() {
    if (!selectedSlot) return;
    setBooking(true);
    const startTime = new Date();

    let durationMinutes = duration;
    if (bookingType === 'mehrtaegig') {
      durationMinutes = differenceInDays(new Date(endDate), new Date(startDate)) * 24 * 60;
    } else if (bookingType === 'dauer') {
      durationMinutes = dauerInterval === 'monthly' ? 30 * 24 * 60 : 7 * 24 * 60;
    }

    const res = await api.createBooking({
      slot_id: selectedSlot.id,
      start_time: bookingType === 'mehrtaegig' ? new Date(startDate).toISOString() : startTime.toISOString(),
      duration_minutes: durationMinutes,
      vehicle_id: selectedVehicle || undefined,
      license_plate: !selectedVehicle ? customPlate : undefined,
    });

    if (res.success) {
      const plate = selectedVehicle ? (vehicles.find(v => v.id === selectedVehicle)?.license_plate || '') : customPlate;
      const typeLabel = bookingType === 'einmalig' ? 'Einmalig' : bookingType === 'mehrtaegig' ? 'Mehrtägig' : 'Dauerbuchung';
      const timeLabel = bookingType === 'einmalig'
        ? `${format(new Date(), 'HH:mm')} – ${format(endTime, 'HH:mm')} Uhr`
        : bookingType === 'mehrtaegig'
        ? `${format(new Date(startDate), 'd. MMM', { locale: de })} – ${format(new Date(endDate), 'd. MMM yyyy', { locale: de })}`
        : `${dauerInterval === 'weekly' ? 'Wöchentlich' : 'Monatlich'}`;
      setSuccessSummary({ lot: selectedLotData?.name || '', slot: selectedSlot!.number, type: typeLabel, time: timeLabel, plate });
      setShowSuccess(true);
    } else {
      toast.error(res.error?.message || 'Buchung fehlgeschlagen');
    }
    setBooking(false);
  }

  const selectedLotData = lots.find(l => l.id === selectedLot);
  const endTime = addMinutes(new Date(), duration);
  const dayNames = ['So', 'Mo', 'Di', 'Mi', 'Do', 'Fr', 'Sa'];

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <SpinnerGap weight="bold" className="w-8 h-8 text-primary-600 animate-spin" />
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Parkplatz buchen</h1>
        <p className="text-gray-500 dark:text-gray-400 mt-1">
          Wählen Sie Ihren Stellplatz und die gewünschte Dauer
        </p>
      </div>

      {/* Step 1: Select Lot */}
      <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="card p-6">
        <div className="flex items-center gap-3 mb-6">
          <div className="w-8 h-8 bg-primary-100 dark:bg-primary-900/30 rounded-lg flex items-center justify-center">
            <span className="text-sm font-bold text-primary-600 dark:text-primary-400">1</span>
          </div>
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Parkplatz wählen</h2>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {lots.map((lot) => (
            <button
              key={lot.id}
              onClick={() => { setSelectedLot(lot.id); setSelectedSlot(null); setDetailedLot(null); }}
              className={`p-4 rounded-xl border-2 text-left transition-all ${
                selectedLot === lot.id
                  ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
                  : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
              }`}
            >
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-3">
                  <MapPin weight="fill" className="w-5 h-5 text-gray-400" />
                  <div>
                    <p className="font-medium text-gray-900 dark:text-white">{lot.name}</p>
                    <p className="text-sm text-gray-500 dark:text-gray-400">{lot.address}</p>
                  </div>
                </div>
                <div className={`badge ${lot.available_slots === 0 ? 'badge-error' : 'badge-success'}`}>
                  {lot.available_slots} frei
                </div>
              </div>
            </button>
          ))}
        </div>
      </motion.div>

      {/* Step 2: Select Slot - ParkingLotGrid */}
      <AnimatePresence>
        {selectedLot && detailedLot?.layout && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
            className="card p-6 shadow-md dark:shadow-gray-900/50"
          >
            <div className="flex items-center gap-3 mb-6">
              <div className="w-8 h-8 bg-primary-100 dark:bg-primary-900/30 rounded-lg flex items-center justify-center">
                <span className="text-sm font-bold text-primary-600 dark:text-primary-400">2</span>
              </div>
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Stellplatz wählen</h2>
              {selectedLotData && (
                <span className="ml-auto text-sm text-gray-500 dark:text-gray-400">
                  {selectedLotData.available_slots} von {selectedLotData.total_slots} verfügbar
                </span>
              )}
            </div>
            
            {selectedSlot && (
              <div className="mb-4 p-3 bg-primary-50 dark:bg-primary-900/20 rounded-xl border border-primary-200 dark:border-primary-800 flex items-center gap-3">
                <CheckCircle weight="fill" className="w-5 h-5 text-primary-600 dark:text-primary-400" />
                <span className="text-sm font-medium text-primary-700 dark:text-primary-300">
                  Stellplatz <strong>{selectedSlot.number}</strong> ausgewählt
                </span>
              </div>
            )}

            <ParkingLotGrid
              layout={detailedLot.layout}
              selectedSlotId={selectedSlot?.id}
              onSlotSelect={handleSlotSelect}
              interactive
            />
          </motion.div>
        )}
      </AnimatePresence>

      {/* Step 3: Duration, Type & Vehicle */}
      <AnimatePresence>
        {selectedSlot && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
            className="card p-6"
          >
            <div className="flex items-center gap-3 mb-6">
              <div className="w-8 h-8 bg-primary-100 dark:bg-primary-900/30 rounded-lg flex items-center justify-center">
                <span className="text-sm font-bold text-primary-600 dark:text-primary-400">3</span>
              </div>
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Dauer & Fahrzeug</h2>
            </div>

            <div className="space-y-6">
              {/* Booking type selector */}
              <div>
                <label className="label flex items-center gap-2">
                  <CalendarBlank weight="regular" className="w-4 h-4" />
                  Buchungsart
                </label>
                <div className="grid grid-cols-3 gap-2">
                  {([
                    { value: 'einmalig' as const, label: 'Einmalig', icon: Clock },
                    { value: 'mehrtaegig' as const, label: 'Mehrtägig', icon: CalendarBlank },
                    { value: 'dauer' as const, label: 'Dauerbuchung', icon: Repeat },
                  ] as const).map(({ value, label, icon: Icon }) => (
                    <button
                      key={value}
                      onClick={() => setBookingType(value)}
                      className={`py-3 px-4 rounded-xl text-sm font-medium transition-all flex items-center justify-center gap-2 ${
                        bookingType === value
                          ? 'bg-primary-600 text-white'
                          : 'bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700'
                      }`}
                    >
                      <Icon weight={bookingType === value ? 'fill' : 'regular'} className="w-4 h-4" />
                      {label}
                    </button>
                  ))}
                </div>
              </div>

              {/* Einmalig: duration buttons */}
              {bookingType === 'einmalig' && (
                <div>
                  <label className="label flex items-center gap-2">
                    <Clock weight="regular" className="w-4 h-4" />
                    Parkdauer
                  </label>
                  <div className="grid grid-cols-3 sm:grid-cols-6 gap-2">
                    {DURATION_OPTIONS.map((opt) => (
                      <button
                        key={opt.value}
                        onClick={() => setDuration(opt.value)}
                        className={`py-2.5 px-4 rounded-xl text-sm font-medium transition-all ${
                          duration === opt.value
                            ? 'bg-primary-600 text-white'
                            : 'bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700'
                        }`}
                      >
                        {opt.label}
                      </button>
                    ))}
                  </div>
                  <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                    Bis {format(endTime, 'HH:mm')} Uhr ({format(endTime, 'EEEE', { locale: de })})
                  </p>
                </div>
              )}

              {/* Mehrtägig: date range picker */}
              {bookingType === 'mehrtaegig' && (
                <div>
                  <label className="label flex items-center gap-2">
                    <CalendarBlank weight="regular" className="w-4 h-4" />
                    Zeitraum
                  </label>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="text-xs text-gray-500 dark:text-gray-400 mb-1 block">Startdatum</label>
                      <input
                        type="date"
                        value={startDate}
                        onChange={(e) => setStartDate(e.target.value)}
                        min={format(new Date(), 'yyyy-MM-dd')}
                        className="input"
                      />
                    </div>
                    <div>
                      <label className="text-xs text-gray-500 dark:text-gray-400 mb-1 block">Enddatum</label>
                      <input
                        type="date"
                        value={endDate}
                        onChange={(e) => setEndDate(e.target.value)}
                        min={startDate}
                        className="input"
                      />
                    </div>
                  </div>
                  {startDate && endDate && (
                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                      {differenceInDays(new Date(endDate), new Date(startDate))} Tage —{' '}
                      {format(new Date(startDate), 'd. MMM', { locale: de })} bis{' '}
                      {format(new Date(endDate), 'd. MMM yyyy', { locale: de })}
                    </p>
                  )}
                </div>
              )}

              {/* Dauerbuchung: weekly/monthly + day selection */}
              {bookingType === 'dauer' && (
                <div className="space-y-4">
                  <div>
                    <label className="label flex items-center gap-2">
                      <Repeat weight="regular" className="w-4 h-4" />
                      Intervall
                    </label>
                    <div className="grid grid-cols-2 gap-2">
                      <button
                        onClick={() => setDauerInterval('weekly')}
                        className={`py-2.5 px-4 rounded-xl text-sm font-medium transition-all ${
                          dauerInterval === 'weekly'
                            ? 'bg-primary-600 text-white'
                            : 'bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700'
                        }`}
                      >
                        Wöchentlich
                      </button>
                      <button
                        onClick={() => setDauerInterval('monthly')}
                        className={`py-2.5 px-4 rounded-xl text-sm font-medium transition-all ${
                          dauerInterval === 'monthly'
                            ? 'bg-primary-600 text-white'
                            : 'bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700'
                        }`}
                      >
                        Monatlich
                      </button>
                    </div>
                  </div>
                  {dauerInterval === 'weekly' && (
                    <div>
                      <label className="text-xs text-gray-500 dark:text-gray-400 mb-2 block">Wochentage</label>
                      <div className="flex gap-2">
                        {dayNames.map((d, i) => (
                          <button
                            key={i}
                            onClick={() => setDauerDays(prev => prev.includes(i) ? prev.filter(x => x !== i) : [...prev, i])}
                            className={`w-10 h-10 rounded-xl text-sm font-medium transition-all ${
                              dauerDays.includes(i)
                                ? 'bg-primary-600 text-white'
                                : 'bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700'
                            }`}
                          >
                            {d}
                          </button>
                        ))}
                      </div>
                    </div>
                  )}
                  <div>
                    <label className="text-xs text-gray-500 dark:text-gray-400 mb-1 block">Startdatum</label>
                    <input
                      type="date"
                      value={startDate}
                      onChange={(e) => setStartDate(e.target.value)}
                      min={format(new Date(), 'yyyy-MM-dd')}
                      className="input max-w-xs"
                    />
                  </div>
                </div>
              )}

              {/* Vehicle */}
              <div>
                <label className="label flex items-center gap-2">
                  <Car weight="regular" className="w-4 h-4" />
                  Fahrzeug
                </label>
                {vehicles.length > 0 && (
                  <select
                    value={selectedVehicle}
                    onChange={(e) => { setSelectedVehicle(e.target.value); setCustomPlate(''); }}
                    className="input"
                  >
                    <option value="">Anderes Kennzeichen eingeben</option>
                    {vehicles.map((v) => (
                      <option key={v.id} value={v.id}>
                        {v.license_plate} {v.make && v.model ? `(${v.make} ${v.model})` : ''}
                      </option>
                    ))}
                  </select>
                )}
                {!selectedVehicle && (
                  <input
                    type="text"
                    value={customPlate}
                    onChange={(e) => setCustomPlate(e.target.value.toUpperCase())}
                    placeholder="Kennzeichen eingeben (z.B. M-AB 1234)"
                    className="input mt-2"
                  />
                )}
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Summary & Book */}
      <AnimatePresence>
        {selectedSlot && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
            className="card bg-gradient-to-br from-primary-600 to-primary-700 p-6 text-white"
          >
            <h2 className="text-lg font-semibold mb-4">Zusammenfassung</h2>
            <div className="grid grid-cols-2 gap-4 mb-6">
              <div>
                <p className="text-white/70 text-sm">Parkplatz</p>
                <p className="font-medium">{selectedLotData?.name}</p>
              </div>
              <div>
                <p className="text-white/70 text-sm">Stellplatz</p>
                <p className="font-medium">{selectedSlot.number}</p>
              </div>
              <div>
                <p className="text-white/70 text-sm">Buchungsart</p>
                <p className="font-medium">
                  {bookingType === 'einmalig' ? 'Einmalig' : bookingType === 'mehrtaegig' ? 'Mehrtägig' : 'Dauerbuchung'}
                </p>
              </div>
              <div>
                <p className="text-white/70 text-sm">
                  {bookingType === 'einmalig' ? 'Dauer' : bookingType === 'mehrtaegig' ? 'Zeitraum' : 'Intervall'}
                </p>
                <p className="font-medium">
                  {bookingType === 'einmalig' && (
                    <>{DURATION_OPTIONS.find(o => o.value === duration)?.label} — bis {format(endTime, 'HH:mm')} Uhr</>
                  )}
                  {bookingType === 'mehrtaegig' && (
                    <>{format(new Date(startDate), 'd. MMM', { locale: de })} — {format(new Date(endDate), 'd. MMM yyyy', { locale: de })}</>
                  )}
                  {bookingType === 'dauer' && (
                    <>{dauerInterval === 'weekly' ? `Wöchentl. (${dauerDays.map(d => dayNames[d]).join(', ')})` : 'Monatlich'} ab {format(new Date(startDate), 'd. MMM yyyy', { locale: de })}</>
                  )}
                </p>
              </div>
              <div className="col-span-2">
                <p className="text-white/70 text-sm">Kennzeichen</p>
                <p className="font-medium">
                  {selectedVehicle ? vehicles.find(v => v.id === selectedVehicle)?.license_plate : customPlate || '—'}
                </p>
              </div>
            </div>
            <button
              onClick={handleBook}
              disabled={booking || (!selectedVehicle && !customPlate)}
              className="btn bg-white text-primary-700 hover:bg-white/90 w-full justify-center"
            >
              {booking ? (
                <SpinnerGap weight="bold" className="w-5 h-5 animate-spin" />
              ) : (
                <>
                  <CheckCircle weight="bold" className="w-5 h-5" />
                  Jetzt buchen
                </>
              )}
            </button>
          </motion.div>
        )}
      </AnimatePresence>
      {/* Success Modal */}
      <BookingSuccessModal
        open={showSuccess}
        summary={successSummary}
        onDashboard={() => navigate('/')}
        onNewBooking={() => {
          setShowSuccess(false);
          setSelectedSlot(null);
          setSelectedLot('');
          setDetailedLot(null);
        }}
      />
    </div>
  );
}
