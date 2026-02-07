import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Car, Prohibit, Lock, House } from '@phosphor-icons/react';
import type { LotLayout, LotRow, SlotConfig } from '../api/client';

interface ParkingLotGridProps {
  layout: LotLayout;
  selectedSlotId?: string;
  onSlotSelect?: (slot: SlotConfig) => void;
  interactive?: boolean;
  vehiclePhotos?: Record<string, string>; // plate -> photoUrl
}

const statusColors: Record<SlotConfig['status'], { bg: string; border: string; text: string }> = {
  available: {
    bg: 'bg-emerald-100 dark:bg-emerald-900/40',
    border: 'border-emerald-300 dark:border-emerald-700',
    text: 'text-emerald-700 dark:text-emerald-300',
  },
  occupied: {
    bg: 'bg-red-100 dark:bg-red-900/40',
    border: 'border-red-300 dark:border-red-700',
    text: 'text-red-700 dark:text-red-300',
  },
  reserved: {
    bg: 'bg-amber-100 dark:bg-amber-900/40',
    border: 'border-amber-300 dark:border-amber-700',
    text: 'text-amber-700 dark:text-amber-300',
  },
  disabled: {
    bg: 'bg-gray-100 dark:bg-gray-800',
    border: 'border-gray-300 dark:border-gray-600 border-dashed',
    text: 'text-gray-400 dark:text-gray-500',
  },
  blocked: {
    bg: 'bg-gray-200 dark:bg-gray-700',
    border: 'border-gray-400 dark:border-gray-500',
    text: 'text-gray-500 dark:text-gray-400',
  },
  homeoffice: {
    bg: 'bg-sky-100 dark:bg-sky-900/30',
    border: 'border-sky-300 dark:border-sky-700',
    text: 'text-sky-700 dark:text-sky-300',
  },
};

function SlotBox({
  slot,
  side,
  selected,
  interactive,
  onSelect,
  vehiclePhoto,
}: {
  slot: SlotConfig;
  side: 'top' | 'bottom';
  selected: boolean;
  interactive: boolean;
  onSelect?: (slot: SlotConfig) => void;
  vehiclePhoto?: string;
}) {
  const [hovered, setHovered] = useState(false);
  const colors = statusColors[slot.status];
  const clickable = interactive && (slot.status === 'available' || slot.status === 'reserved' || slot.status === 'homeoffice');
  const tooltip = slot.status === 'homeoffice' && slot.homeofficeUser ? `Frei (Homeoffice von ${slot.homeofficeUser})` : undefined;

  return (
    <motion.div
      className="relative flex-shrink-0"
      whileHover={clickable ? { scale: 1.05 } : {}}
      whileTap={clickable ? { scale: 0.97 } : {}}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {/* Hover tooltip for occupied slots */}
      <AnimatePresence>
        {hovered && slot.status === 'occupied' && slot.vehiclePlate && (
          <motion.div
            initial={{ opacity: 0, y: 4 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 4 }}
            className={`absolute z-20 ${side === 'top' ? 'top-full mt-1' : 'bottom-full mb-1'} left-1/2 -translate-x-1/2 bg-gray-900 dark:bg-gray-700 text-white text-xs rounded-lg px-3 py-2 whitespace-nowrap shadow-lg flex items-center gap-2`}
          >
            {vehiclePhoto && <img src={vehiclePhoto} alt="" className="w-8 h-8 rounded-full object-cover" />}
            <span>Belegt: {slot.vehiclePlate}</span>
          </motion.div>
        )}
      </AnimatePresence>
      <button
        disabled={!clickable}
        onClick={() => clickable && onSelect?.(slot)}
        title={tooltip}
        className={`
          w-20 h-24 sm:w-24 sm:h-28 rounded-xl border-2 flex flex-col items-center justify-center gap-0.5 transition-all shadow-sm
          ${colors.bg} ${colors.border} ${colors.text}
          ${clickable ? 'cursor-pointer hover:shadow-md hover:brightness-105' : 'cursor-default'}
          ${selected ? 'ring-2 ring-primary-500 ring-offset-2 dark:ring-offset-gray-900 shadow-lg shadow-primary-500/20' : ''}
          ${slot.status === 'disabled' ? 'opacity-60' : ''}
        `}
      >
        {/* Car icon pointing toward road */}
        {slot.status === 'occupied' && (
          <Car
            weight="fill"
            className={`w-6 h-6 sm:w-7 sm:h-7 ${side === 'top' ? 'rotate-180' : ''}`}
          />
        )}
        {slot.status === 'homeoffice' && (
          <House weight="fill" className="w-6 h-6 sm:w-7 sm:h-7" />
        )}
        {slot.status === 'disabled' && <Prohibit weight="bold" className="w-5 h-5" />}
        {slot.status === 'blocked' && <Lock weight="fill" className="w-5 h-5" />}
        {(slot.status === 'available' || slot.status === 'reserved') && (
          <Car
            weight="regular"
            className={`w-6 h-6 sm:w-7 sm:h-7 opacity-30 ${side === 'top' ? 'rotate-180' : ''}`}
          />
        )}
        <span className="text-base sm:text-lg font-extrabold leading-tight">{slot.number}</span>
        {/* License plate directly on occupied slots */}
        {slot.status === 'occupied' && slot.vehiclePlate && (
          <span className="text-[9px] sm:text-[10px] font-mono opacity-75 leading-none truncate max-w-[4.5rem] sm:max-w-[5.5rem]">
            {slot.vehiclePlate}
          </span>
        )}
        {slot.status === 'homeoffice' && (
          <span className="text-[9px] sm:text-[10px] font-semibold opacity-75 leading-none">HO</span>
        )}
      </button>
    </motion.div>
  );
}

function RowSlots({
  row,
  selectedSlotId,
  interactive,
  onSlotSelect,
  vehiclePhotos,
}: {
  row: LotRow;
  selectedSlotId?: string;
  interactive: boolean;
  onSlotSelect?: (slot: SlotConfig) => void;
  vehiclePhotos?: Record<string, string>;
}) {
  return (
    <div className="flex flex-col gap-1">
      {row.label && (
        <span className="text-[10px] font-medium text-gray-300 dark:text-gray-600 uppercase tracking-widest px-1 select-none">
          {row.label}
        </span>
      )}
      <div className="flex gap-2 sm:gap-2.5">
        {row.slots.map((slot) => (
          <SlotBox
            key={slot.id}
            slot={slot}
            side={row.side}
            selected={slot.id === selectedSlotId}
            interactive={interactive}
            onSelect={onSlotSelect}
            vehiclePhoto={slot.vehiclePlate ? vehiclePhotos?.[slot.vehiclePlate] : undefined}
          />
        ))}
      </div>
    </div>
  );
}

export function ParkingLotGrid({
  layout,
  selectedSlotId,
  onSlotSelect,
  interactive = false,
  vehiclePhotos,
}: ParkingLotGridProps) {
  const topRows = layout.rows.filter((r) => r.side === 'top');
  const bottomRows = layout.rows.filter((r) => r.side === 'bottom');

  return (
    <div className="space-y-3">
      {/* Grid */}
      <div className="overflow-x-auto pb-2">
        <div className="inline-flex flex-col gap-0 min-w-fit">
          {/* Top rows */}
          {topRows.map((row) => (
            <RowSlots
              key={row.id}
              row={row}
              selectedSlotId={selectedSlotId}
              interactive={interactive}
              onSlotSelect={onSlotSelect}
              vehiclePhotos={vehiclePhotos}
            />
          ))}

          {/* Road */}
          <div className="my-2 rounded-md bg-gray-200/60 dark:bg-gray-800 py-2 px-4 flex items-center gap-3">
            <div className="flex-1 border-t border-dashed border-gray-300 dark:border-gray-700" />
            <span className="text-[10px] font-medium text-gray-400 dark:text-gray-600 uppercase tracking-[0.2em] select-none">
              {layout.roadLabel || 'Fahrweg'}
            </span>
            <div className="flex-1 border-t border-dashed border-gray-300 dark:border-gray-700" />
          </div>

          {/* Bottom rows */}
          {bottomRows.map((row) => (
            <RowSlots
              key={row.id}
              row={row}
              selectedSlotId={selectedSlotId}
              interactive={interactive}
              onSlotSelect={onSlotSelect}
              vehiclePhotos={vehiclePhotos}
            />
          ))}
        </div>
      </div>

      {/* Legend */}
      <div className="flex flex-wrap gap-4 text-xs text-gray-500 dark:text-gray-400 pt-2 border-t border-gray-100 dark:border-gray-800">
        {[
          { status: 'available' as const, label: 'Frei' },
          { status: 'occupied' as const, label: 'Belegt' },
          { status: 'reserved' as const, label: 'Reserviert' },
          { status: 'disabled' as const, label: 'Gesperrt' },
          { status: 'homeoffice' as const, label: '🏠 Homeoffice (frei)' },
        ].map(({ status, label }) => (
          <div key={status} className="flex items-center gap-1.5">
            <div
              className={`w-3 h-3 rounded-sm border ${statusColors[status].bg} ${statusColors[status].border}`}
            />
            <span>{label}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
