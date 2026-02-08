import { useState, useEffect, createContext, useContext, ReactNode } from 'react';
import { SpinnerGap } from '@phosphor-icons/react';

interface SetupContextType {
  setupComplete: boolean | null;
  recheckSetup: () => Promise<void>;
}

const SetupContext = createContext<SetupContextType>({ setupComplete: null, recheckSetup: async () => {} });

export function useSetupStatus() {
  return useContext(SetupContext);
}

export function SetupGuard({ children }: { children: ReactNode }) {
  const [setupComplete, setSetupComplete] = useState<boolean | null>(null);

  async function checkSetup() {
    try {
      const res = await fetch('/api/v1/setup/status');
      const data = await res.json();
      if (data.success && data.data) {
        setSetupComplete(!!data.data.setup_complete);
      } else {
        setSetupComplete(true); // Assume complete on error
      }
    } catch {
      setSetupComplete(true);
    }
  }

  useEffect(() => {
    checkSetup();
  }, []);

  if (setupComplete === null) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-950">
        <SpinnerGap weight="bold" className="w-8 h-8 text-primary-600 animate-spin" />
      </div>
    );
  }

  return (
    <SetupContext.Provider value={{ setupComplete, recheckSetup: checkSetup }}>
      {children}
    </SetupContext.Provider>
  );
}
