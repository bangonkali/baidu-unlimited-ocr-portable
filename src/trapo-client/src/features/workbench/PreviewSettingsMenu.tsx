import * as DropdownMenu from '@radix-ui/react-dropdown-menu';
import { Check, ChevronRight, FileText, ScanText, Settings } from 'lucide-react';

import type { IngestPreviewResultRecord } from '../../api/types';
import styles from './PreviewPane.module.css';

interface PreviewSettingsMenuProps {
  autoFollowRegions: boolean;
  engineResults: IngestPreviewResultRecord[];
  selectedRunEngineId?: string;
  onAutoFollowChange: (enabled: boolean) => void;
  onSelectPreviewResult: (runEngineId: string) => void;
}

export function PreviewSettingsMenu({
  autoFollowRegions,
  engineResults,
  selectedRunEngineId,
  onAutoFollowChange,
  onSelectPreviewResult,
}: PreviewSettingsMenuProps) {
  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger asChild>
        <button className={styles.settingsButton} type="button">
          <Settings size={14} strokeWidth={1.9} />
          <span>Settings</span>
        </button>
      </DropdownMenu.Trigger>
      <DropdownMenu.Portal>
        <DropdownMenu.Content align="end" className={styles.menu} sideOffset={4}>
          <DropdownMenu.CheckboxItem
            checked={autoFollowRegions}
            className={styles.menuItem}
            onCheckedChange={(checked) => onAutoFollowChange(checked === true)}
          >
            <ItemCheck />
            <span>Auto Follow</span>
          </DropdownMenu.CheckboxItem>
          <DropdownMenu.Separator className={styles.menuSeparator} />
          <EngineSubmenu
            engineResults={engineResults}
            selectedRunEngineId={selectedRunEngineId}
            onSelectPreviewResult={onSelectPreviewResult}
          />
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
}

function EngineSubmenu({
  engineResults,
  selectedRunEngineId,
  onSelectPreviewResult,
}: Pick<
  PreviewSettingsMenuProps,
  'engineResults' | 'selectedRunEngineId' | 'onSelectPreviewResult'
>) {
  return (
    <DropdownMenu.Sub>
      <DropdownMenu.SubTrigger className={styles.menuItem} disabled={engineResults.length === 0}>
        <span className={styles.menuIconSlot} />
        <span>Engine</span>
        <ChevronRight className={styles.menuArrow} size={13} strokeWidth={1.9} />
      </DropdownMenu.SubTrigger>
      <DropdownMenu.Portal>
        <DropdownMenu.SubContent className={styles.menu} sideOffset={3}>
          {engineResults.map((result) => (
            <DropdownMenu.Item
              className={styles.menuItem}
              key={result.run_engine_id}
              onSelect={() => onSelectPreviewResult(result.run_engine_id)}
            >
              <span className={styles.menuIconSlot}>
                {result.run_engine_id === selectedRunEngineId ? <Check size={13} /> : null}
              </span>
              {result.previewer === 'document_markdown' ? (
                <FileText size={13} strokeWidth={1.9} />
              ) : (
                <ScanText size={13} strokeWidth={1.9} />
              )}
              <span className={styles.engineLabel}>{result.label}</span>
              <span className={styles.engineStatus}>{statusLabel(result.status)}</span>
            </DropdownMenu.Item>
          ))}
        </DropdownMenu.SubContent>
      </DropdownMenu.Portal>
    </DropdownMenu.Sub>
  );
}

function ItemCheck() {
  return (
    <span className={styles.menuIconSlot}>
      <DropdownMenu.ItemIndicator>
        <Check size={13} strokeWidth={1.9} />
      </DropdownMenu.ItemIndicator>
    </span>
  );
}

function statusLabel(status: string) {
  switch (status) {
    case 'completed':
      return 'ready';
    case 'completed_with_errors':
      return 'partial';
    default:
      return status.replaceAll('_', ' ');
  }
}
