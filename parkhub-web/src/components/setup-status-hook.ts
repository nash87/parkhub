import { useContext } from 'react';
import { SetupContext } from './SetupGuard';

export function useSetupStatus() {
  return useContext(SetupContext);
}
