import type { PointerEvent } from 'react';

import styles from './DiagnosticsWaterfallControls.module.css';

interface ResizeHandleProps {
  label: string;
  onPointerDown: (event: PointerEvent<HTMLButtonElement>) => void;
}

export function ResizeHandle({ label, onPointerDown }: ResizeHandleProps) {
  return (
    <button
      aria-label={label}
      className={styles.resizeHandle}
      onPointerDown={onPointerDown}
      title={label}
      type="button"
    />
  );
}
