import { ReactNode, useState, useEffect, useRef } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';
import {
  House,
  CalendarPlus,
  ListChecks,
  Car,
  GearSix,
  SignOut,
  Moon,
  Sun,
  List,
  X,
  Bell,
  User,
  CaretDown,
  Warning,
  Info,
  CheckCircle,
  DotsThreeCircle,
} from '@phosphor-icons/react';
import { useAuth } from '../context/AuthContext';
import { useTheme, applyTheme } from '../stores/theme';

interface LayoutProps {
  children: ReactNode;
}

const navigation = [
  { name: 'Dashboard', href: '/', icon: House },
  { name: 'Buchen', href: '/book', icon: CalendarPlus },
  { name: 'Buchungen', href: '/bookings', icon: ListChecks },
  { name: 'Fahrzeuge', href: '/vehicles', icon: Car },
  { name: 'Homeoffice', href: '/homeoffice', icon: House },
];

const adminNav = [
  { name: 'Admin', href: '/admin', icon: GearSix },
];

interface Notification {
  id: string;
  text: string;
  type: 'warning' | 'info' | 'success';
  read: boolean;
}

const initialNotifications: Notification[] = [
  { id: 'n1', text: 'Ihre Buchung für Stellplatz 47 läuft in 30 Min ab', type: 'warning', read: false },
  { id: 'n2', text: "Neuer Parkplatz 'Tiefgarage Süd' verfügbar", type: 'info', read: false },
  { id: 'n3', text: 'Stellplatz 52 wurde für Sie freigegeben (Homeoffice)', type: 'success', read: false },
];

const notifIcon = { warning: Warning, info: Info, success: CheckCircle };
const notifColor = {
  warning: 'text-amber-500',
  info: 'text-blue-500',
  success: 'text-emerald-500',
};

export function Layout({ children }: LayoutProps) {
  const { user, logout } = useAuth();
  const location = useLocation();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [userMenuOpen, setUserMenuOpen] = useState(false);
  const [notifOpen, setNotifOpen] = useState(false);
  const [notifications, setNotifications] = useState<Notification[]>(initialNotifications);
  const { isDark, toggle } = useTheme();
  const notifRef = useRef<HTMLDivElement>(null);

  const isAdmin = user?.role === 'admin' || user?.role === 'superadmin';
  const unreadCount = notifications.filter(n => !n.read).length;

  useEffect(() => { applyTheme(isDark); }, [isDark]);

  useEffect(() => {
    setMobileMenuOpen(false);
    setUserMenuOpen(false);
    setNotifOpen(false);
  }, [location.pathname]);

  // Close dropdowns on outside click
  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (notifRef.current && !notifRef.current.contains(e.target as Node)) setNotifOpen(false);
    }
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, []);

  function markAsRead(id: string) {
    setNotifications(prev => prev.map(n => n.id === id ? { ...n, read: true } : n));
  }

  return (
    <div className="min-h-screen flex flex-col">
      {/* Header */}
      <header className="sticky top-0 z-50 bg-white/80 dark:bg-gray-900/80 backdrop-blur-lg border-b border-gray-100 dark:border-gray-800">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <Link to="/" className="flex items-center gap-3">
              <div className="w-9 h-9 bg-primary-600 rounded-xl flex items-center justify-center">
                <Car weight="fill" className="w-5 h-5 text-white" />
              </div>
              <span className="text-lg font-bold text-gray-900 dark:text-white">ParkHub</span>
            </Link>

            <nav className="hidden md:flex items-center gap-1">
              {navigation.map((item) => {
                const Icon = item.icon;
                const isActive = location.pathname === item.href;
                return (
                  <Link key={item.href} to={item.href} className={`flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-medium transition-colors ${isActive ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/30 dark:text-primary-400' : 'text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800'}`}>
                    <Icon weight={isActive ? 'fill' : 'regular'} className="w-5 h-5" />
                    {item.name}
                  </Link>
                );
              })}
              {isAdmin && adminNav.map((item) => {
                const Icon = item.icon;
                const isActive = location.pathname.startsWith(item.href);
                return (
                  <Link key={item.href} to={item.href} className={`flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-medium transition-colors ${isActive ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/30 dark:text-primary-400' : 'text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800'}`}>
                    <Icon weight={isActive ? 'fill' : 'regular'} className="w-5 h-5" />
                    {item.name}
                  </Link>
                );
              })}
            </nav>

            <div className="flex items-center gap-2">
              <button onClick={toggle} className="btn btn-ghost btn-icon" aria-label="Theme umschalten">
                {isDark ? <Sun weight="fill" className="w-5 h-5" /> : <Moon weight="fill" className="w-5 h-5" />}
              </button>

              {/* Notifications */}
              <div ref={notifRef} className="relative">
                <button
                  onClick={() => setNotifOpen(!notifOpen)}
                  className="btn btn-ghost btn-icon relative"
                >
                  <Bell weight={notifOpen ? 'fill' : 'regular'} className="w-5 h-5" />
                  {unreadCount > 0 && (
                    <motion.span
                      initial={{ scale: 0 }}
                      animate={{ scale: 1 }}
                      className="absolute -top-0.5 -right-0.5 w-5 h-5 bg-red-500 text-white text-[10px] font-bold rounded-full flex items-center justify-center"
                    >
                      {unreadCount}
                    </motion.span>
                  )}
                </button>

                <AnimatePresence>
                  {notifOpen && (
                    <motion.div
                      initial={{ opacity: 0, y: 10, scale: 0.95 }}
                      animate={{ opacity: 1, y: 0, scale: 1 }}
                      exit={{ opacity: 0, y: 10, scale: 0.95 }}
                      transition={{ type: 'spring', damping: 25, stiffness: 300 }}
                      className="absolute right-0 mt-2 w-80 card p-0 shadow-lg overflow-hidden"
                    >
                      <div className="px-4 py-3 border-b border-gray-100 dark:border-gray-800">
                        <p className="font-semibold text-gray-900 dark:text-white text-sm">Benachrichtigungen</p>
                      </div>
                      <div className="max-h-72 overflow-y-auto">
                        {notifications.map((n) => {
                          const NIcon = notifIcon[n.type];
                          return (
                            <motion.button
                              key={n.id}
                              initial={{ opacity: 0, x: -10 }}
                              animate={{ opacity: 1, x: 0 }}
                              onClick={() => markAsRead(n.id)}
                              className={`w-full text-left px-4 py-3 flex items-start gap-3 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors border-b border-gray-50 dark:border-gray-800/50 ${n.read ? 'opacity-60' : ''}`}
                            >
                              <NIcon weight="fill" className={`w-5 h-5 mt-0.5 flex-shrink-0 ${notifColor[n.type]}`} />
                              <div className="flex-1 min-w-0">
                                <p className={`text-sm ${n.read ? 'text-gray-500 dark:text-gray-400' : 'text-gray-900 dark:text-white font-medium'}`}>
                                  {n.text}
                                </p>
                                <p className="text-xs text-gray-400 mt-0.5">vor 5 Min</p>
                              </div>
                              {!n.read && <span className="w-2 h-2 bg-primary-500 rounded-full mt-2 flex-shrink-0" />}
                            </motion.button>
                          );
                        })}
                      </div>
                      {notifications.length === 0 && (
                        <div className="p-8 text-center">
                          <Bell weight="light" className="w-10 h-10 text-gray-300 dark:text-gray-600 mx-auto mb-2" />
                          <p className="text-sm text-gray-400">Keine Benachrichtigungen</p>
                        </div>
                      )}
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>

              {/* User Menu */}
              <div className="relative hidden md:block">
                <button onClick={() => setUserMenuOpen(!userMenuOpen)} className="flex items-center gap-2 p-1.5 pr-3 rounded-xl hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">
                  <div className="avatar text-sm">{user?.name?.charAt(0).toUpperCase()}</div>
                  <span className="text-sm font-medium text-gray-700 dark:text-gray-300">{user?.name?.split(' ')[0]}</span>
                  <CaretDown weight="bold" className="w-4 h-4 text-gray-400" />
                </button>

                <AnimatePresence>
                  {userMenuOpen && (
                    <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: 10 }} className="absolute right-0 mt-2 w-56 card p-2 shadow-lg">
                      <div className="px-3 py-2 border-b border-gray-100 dark:border-gray-800 mb-2">
                        <p className="font-medium text-gray-900 dark:text-white">{user?.name}</p>
                        <p className="text-sm text-gray-500">{user?.email}</p>
                      </div>
                      <Link to="/profile" className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800">
                        <User weight="regular" className="w-4 h-4" /> Profil
                      </Link>
                      <Link to="/settings" className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800">
                        <GearSix weight="regular" className="w-4 h-4" /> Einstellungen
                      </Link>
                      <button onClick={logout} className="flex items-center gap-2 w-full px-3 py-2 rounded-lg text-sm text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20">
                        <SignOut weight="regular" className="w-4 h-4" /> Abmelden
                      </button>
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>

              <button onClick={() => setMobileMenuOpen(!mobileMenuOpen)} className="md:hidden btn btn-ghost btn-icon">
                {mobileMenuOpen ? <X weight="bold" className="w-5 h-5" /> : <List weight="bold" className="w-5 h-5" />}
              </button>
            </div>
          </div>
        </div>

        {/* Mobile Navigation */}
        <AnimatePresence>
          {mobileMenuOpen && (
            <motion.div initial={{ height: 0, opacity: 0 }} animate={{ height: 'auto', opacity: 1 }} exit={{ height: 0, opacity: 0 }} className="md:hidden overflow-hidden border-t border-gray-100 dark:border-gray-800">
              <div className="px-4 py-3 space-y-1">
                {navigation.map((item) => {
                  const Icon = item.icon;
                  const isActive = location.pathname === item.href;
                  return (
                    <Link key={item.href} to={item.href} className={`flex items-center gap-3 px-4 py-3 rounded-xl text-base font-medium ${isActive ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/30 dark:text-primary-400' : 'text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800'}`}>
                      <Icon weight={isActive ? 'fill' : 'regular'} className="w-5 h-5" /> {item.name}
                    </Link>
                  );
                })}
                {isAdmin && adminNav.map((item) => {
                  const Icon = item.icon;
                  return (
                    <Link key={item.href} to={item.href} className="flex items-center gap-3 px-4 py-3 rounded-xl text-base font-medium text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800">
                      <Icon weight="regular" className="w-5 h-5" /> {item.name}
                    </Link>
                  );
                })}
                <div className="pt-3 border-t border-gray-100 dark:border-gray-800">
                  <button onClick={logout} className="flex items-center gap-3 w-full px-4 py-3 rounded-xl text-base font-medium text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20">
                    <SignOut weight="regular" className="w-5 h-5" /> Abmelden
                  </button>
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </header>

      {/* Main Content */}
      <main className="flex-1">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <motion.div key={location.pathname} initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.2 }}>
            {children}
          </motion.div>
        </div>
      </main>

      {/* Footer - hidden on mobile when bottom bar shows */}
      <footer className="hidden md:block bg-white dark:bg-gray-900 border-t border-gray-100 dark:border-gray-800">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <div className="flex flex-col sm:flex-row items-center justify-between gap-3">
            <p className="text-sm text-gray-500 dark:text-gray-400">
              ParkHub — Open Source Parking Management · <span className="text-gray-400 dark:text-gray-500">v0.1.0</span>
            </p>
            <div className="flex items-center gap-4 text-sm text-gray-400 dark:text-gray-500">
              <a href="#" className="hover:text-gray-600 dark:hover:text-gray-300 transition-colors">Hilfe</a>
              <a href="#" className="hover:text-gray-600 dark:hover:text-gray-300 transition-colors">Datenschutz</a>
              <a href="#" className="hover:text-gray-600 dark:hover:text-gray-300 transition-colors">Impressum</a>
            </div>
          </div>
        </div>
      </footer>

      {/* Mobile Bottom Tab Bar */}
      <MobileBottomBar isAdmin={isAdmin} />

      {/* Bottom padding for mobile to account for bottom bar */}
      <div className="h-16 md:hidden" />
    </div>
  );
}

const mobileTabItems = [
  { name: 'Dashboard', href: '/', icon: House },
  { name: 'Buchen', href: '/book', icon: CalendarPlus },
  { name: 'Buchungen', href: '/bookings', icon: ListChecks },
  { name: 'Homeoffice', href: '/homeoffice', icon: House },
  { name: 'Mehr', href: '#more', icon: DotsThreeCircle },
];

const moreItems = [
  { name: 'Fahrzeuge', href: '/vehicles', icon: Car },
  { name: 'Profil', href: '/profile', icon: User },
  { name: 'Admin', href: '/admin', icon: GearSix, adminOnly: true },
];

function MobileBottomBar({ isAdmin }: { isAdmin: boolean }) {
  const location = useLocation();
  const [showMore, setShowMore] = useState(false);

  return (
    <>
      {/* More Sheet */}
      <AnimatePresence>
        {showMore && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="fixed inset-0 z-40 md:hidden" onClick={() => setShowMore(false)}>
            <div className="absolute inset-0 bg-black/30" />
            <motion.div
              initial={{ y: '100%' }}
              animate={{ y: 0 }}
              exit={{ y: '100%' }}
              transition={{ type: 'spring', damping: 25, stiffness: 300 }}
              className="absolute bottom-16 left-0 right-0 bg-white dark:bg-gray-900 rounded-t-2xl border-t border-gray-200 dark:border-gray-800 p-4 shadow-2xl"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="w-8 h-1 bg-gray-300 dark:bg-gray-700 rounded-full mx-auto mb-4" />
              <div className="space-y-1">
                {moreItems.filter(i => !i.adminOnly || isAdmin).map((item) => {
                  const Icon = item.icon;
                  const isActive = location.pathname === item.href;
                  return (
                    <Link
                      key={item.href}
                      to={item.href}
                      onClick={() => setShowMore(false)}
                      className={`flex items-center gap-3 px-4 py-3 rounded-xl text-base font-medium ${isActive ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/30 dark:text-primary-400' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800'}`}
                    >
                      <Icon weight={isActive ? 'fill' : 'regular'} className="w-5 h-5" />
                      {item.name}
                    </Link>
                  );
                })}
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Bottom Tab Bar */}
      <nav className="fixed bottom-0 left-0 right-0 z-30 md:hidden bg-white/90 dark:bg-gray-900/90 backdrop-blur-lg border-t border-gray-200 dark:border-gray-800 safe-area-bottom">
        <div className="flex items-center justify-around h-16">
          {mobileTabItems.map((item) => {
            const Icon = item.icon;
            const isMore = item.href === '#more';
            const isActive = !isMore && location.pathname === item.href;
            const isMoreActive = isMore && moreItems.some(m => location.pathname === m.href);

            if (isMore) {
              return (
                <button
                  key="more"
                  onClick={() => setShowMore(!showMore)}
                  className={`flex flex-col items-center justify-center gap-0.5 w-16 h-full text-xs font-medium transition-colors ${isMoreActive || showMore ? 'text-primary-600 dark:text-primary-400' : 'text-gray-400 dark:text-gray-500'}`}
                >
                  <Icon weight={isMoreActive || showMore ? 'fill' : 'regular'} className="w-6 h-6" />
                  <span>{item.name}</span>
                </button>
              );
            }

            return (
              <Link
                key={item.href}
                to={item.href}
                onClick={() => setShowMore(false)}
                className={`flex flex-col items-center justify-center gap-0.5 w-16 h-full text-xs font-medium transition-colors ${isActive ? 'text-primary-600 dark:text-primary-400' : 'text-gray-400 dark:text-gray-500'}`}
              >
                <Icon weight={isActive ? 'fill' : 'regular'} className="w-6 h-6" />
                <span>{item.name}</span>
              </Link>
            );
          })}
        </div>
      </nav>
    </>
  );
}
