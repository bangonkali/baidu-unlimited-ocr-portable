import { useEffect, useState } from 'react';

export const FOCUS_EMPHASIS_MS = 1400;

export function useFocusEmphasis(key: string | number | undefined, revision = 0) {
  const [activeKey, setActiveKey] = useState<string | undefined>(undefined);

  useEffect(() => {
    if (key === undefined || revision <= 0) {
      setActiveKey(undefined);
      return undefined;
    }
    const nextKey = String(key);
    setActiveKey(nextKey);
    const timeoutId = window.setTimeout(() => setActiveKey(undefined), FOCUS_EMPHASIS_MS);
    return () => window.clearTimeout(timeoutId);
  }, [key, revision]);

  return activeKey;
}
