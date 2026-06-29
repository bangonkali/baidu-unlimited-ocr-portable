import type { ComponentType } from 'react';

import styles from './IconButton.module.css';

interface IconButtonProps {
  icon: ComponentType<{ size?: number; strokeWidth?: number }>;
  label: string;
  disabled?: boolean;
  pressed?: boolean;
  onClick?: () => void;
}

export function IconButton({ disabled, icon: Icon, label, onClick, pressed }: IconButtonProps) {
  return (
    <button
      aria-label={label}
      aria-pressed={pressed}
      className={styles.button}
      disabled={disabled}
      onClick={onClick}
      title={label}
      type="button"
    >
      <Icon size={17} strokeWidth={1.8} />
    </button>
  );
}
