import { useState, useRef, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Warning } from '@phosphor-icons/react';

const PLATE_REGEX = /^[A-ZÄÖÜ]{1,3}-[A-Z]{1,2}\s\d{1,4}$/;

function formatPlate(raw: string): string {
  const upper = raw.toUpperCase().replace(/[^A-ZÄÖÜ0-9]/g, '');
  let result = '';
  let phase: 'city' | 'letters' | 'numbers' = 'city';
  let cityLen = 0;
  let letterLen = 0;

  for (const ch of upper) {
    if (phase === 'city') {
      if (/[A-ZÄÖÜ]/.test(ch) && cityLen < 3) {
        result += ch;
        cityLen++;
      } else if (/[A-ZÄÖÜ]/.test(ch) && cityLen >= 1) {
        result += '-' + ch;
        letterLen = 1;
        phase = 'letters';
      } else if (/\d/.test(ch) && cityLen >= 1) {
        result += '- ' + ch;
        phase = 'numbers';
      }
    } else if (phase === 'letters') {
      if (/[A-Z]/.test(ch) && letterLen < 2) {
        result += ch;
        letterLen++;
      } else if (/\d/.test(ch)) {
        result += ' ' + ch;
        phase = 'numbers';
      }
    } else if (phase === 'numbers') {
      if (/\d/.test(ch) && (result.match(/\d/g) || []).length < 4) {
        result += ch;
      }
    }
  }
  return result;
}

interface LicensePlateInputProps {
  value: string;
  onChange: (value: string) => void;
  className?: string;
  required?: boolean;
  autoFocus?: boolean;
}

export function LicensePlateInput({ value, onChange, className = '', required, autoFocus }: LicensePlateInputProps) {
  const { t } = useTranslation();
  const [error, setError] = useState('');
  const [touched, setTouched] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const handleChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const formatted = formatPlate(e.target.value);
    onChange(formatted);
    if (touched && formatted && !PLATE_REGEX.test(formatted)) {
      setError(t('vehicle.formatHint'));
    } else {
      setError('');
    }
  }, [onChange, touched, t]);

  const handleBlur = useCallback(() => {
    setTouched(true);
    if (value && !PLATE_REGEX.test(value)) {
      setError(t('vehicle.formatHint'));
    } else {
      setError('');
    }
  }, [value, t]);

  return (
    <div>
      <div className="relative">
        <input
          ref={inputRef}
          type="text"
          value={value}
          onChange={handleChange}
          onBlur={handleBlur}
          placeholder="GÖ-AB 1234"
          className={`input font-mono text-lg tracking-wider ${error ? 'border-red-400 dark:border-red-500' : ''} ${className}`}
          required={required}
          autoFocus={autoFocus}
          autoCapitalize="characters"
          autoComplete="off"
          inputMode="text"
        />
      </div>
      <div className="flex items-center justify-between mt-1.5">
        <span className="text-xs text-gray-400 dark:text-gray-500 font-mono">{t('vehicle.formatHint')}</span>
        {error && touched && (
          <span className="text-xs text-red-500 flex items-center gap-1">
            <Warning weight="bold" className="w-3 h-3" />
            {error}
          </span>
        )}
      </div>
    </div>
  );
}
