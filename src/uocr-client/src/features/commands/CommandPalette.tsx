import {
  Activity,
  Boxes,
  Download,
  FolderOpen,
  LayoutPanelTop,
  Moon,
  Route,
  Search,
  Settings2,
  Sun,
} from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';

import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
  CommandShortcut,
} from '../../components/ui/command';
import type { AppCommand, AppCommandAction, CommandIconName } from './appCommands';
import styles from './CommandPalette.module.css';

interface CommandPaletteProps {
  commands: AppCommand[];
  open: boolean;
  onExecute: (action: AppCommandAction) => void;
  onOpenChange: (open: boolean) => void;
}

export function CommandPalette({ commands, onExecute, onOpenChange, open }: CommandPaletteProps) {
  const [query, setQuery] = useState('');
  const [notice, setNotice] = useState('');

  useEffect(() => {
    if (open) {
      setQuery('');
      setNotice('');
    }
  }, [open]);

  const grouped = useMemo(() => groupCommands(commands), [commands]);
  const trimmedQuery = query.trim();

  if (!open) {
    return null;
  }

  return (
    <div className={styles.overlay}>
      <button
        aria-label="Close command center"
        className={styles.backdrop}
        onClick={() => onOpenChange(false)}
        type="button"
      />
      <div aria-modal="true" className={styles.dialog} role="dialog">
        <Command shouldFilter>
          <div className={styles.inputRow}>
            <Search size={15} />
            <CommandInput
              autoFocus
              onKeyDown={(event) => {
                if (event.key === 'Escape') {
                  onOpenChange(false);
                }
              }}
              onValueChange={setQuery}
              placeholder="Search commands, routes, models, or documents"
              value={query}
            />
          </div>
          <CommandList>
            <CommandEmpty>No commands found.</CommandEmpty>
            {trimmedQuery ? (
              <>
                <CommandGroup heading="Search">
                  <CommandItem
                    onSelect={() => {
                      onExecute({ kind: 'filterDocuments', q: trimmedQuery });
                      onOpenChange(false);
                    }}
                    value={`filter documents ${trimmedQuery}`}
                  >
                    <Search size={14} />
                    <CommandText
                      description="Filter the Workbench document explorer by this text."
                      label={`Search documents for "${trimmedQuery}"`}
                    />
                    <CommandShortcut>Enter</CommandShortcut>
                  </CommandItem>
                </CommandGroup>
                <CommandSeparator />
              </>
            ) : null}
            {grouped.map(([group, items]) => (
              <CommandGroup heading={group} key={group}>
                {items.map((command) => (
                  <CommandItem
                    disabled={command.disabled}
                    key={command.id}
                    onSelect={() => {
                      if (command.disabled) {
                        setNotice(command.description);
                        return;
                      }
                      onExecute(command.action);
                      onOpenChange(false);
                    }}
                    value={commandValue(command)}
                  >
                    <CommandIcon icon={command.icon} />
                    <CommandText description={command.description} label={command.label} />
                    {command.shortcut ? (
                      <CommandShortcut>{command.shortcut}</CommandShortcut>
                    ) : null}
                  </CommandItem>
                ))}
              </CommandGroup>
            ))}
          </CommandList>
        </Command>
        {notice ? <div className={styles.notice}>{notice}</div> : null}
      </div>
    </div>
  );
}

function CommandText({ description, label }: { description: string; label: string }) {
  return (
    <div className={styles.resultText}>
      <strong>{label}</strong>
      <span>{description}</span>
    </div>
  );
}

function CommandIcon({ icon }: { icon: CommandIconName }) {
  const Icon = iconComponent(icon);
  return <Icon size={14} strokeWidth={1.9} />;
}

function groupCommands(commands: AppCommand[]): Array<[string, AppCommand[]]> {
  const groups = new Map<string, AppCommand[]>();
  for (const command of commands) {
    const existing = groups.get(command.group) ?? [];
    existing.push(command);
    groups.set(command.group, existing);
  }
  return Array.from(groups.entries());
}

function commandValue(command: AppCommand) {
  return [command.label, command.description, ...command.keywords].join(' ');
}

function iconComponent(icon: CommandIconName) {
  switch (icon) {
    case 'diagnostics':
      return Activity;
    case 'download':
      return Download;
    case 'folder':
      return FolderOpen;
    case 'layout':
      return LayoutPanelTop;
    case 'model':
      return Boxes;
    case 'settings':
      return Settings2;
    case 'theme':
      return documentThemeIcon();
    case 'route':
      return Route;
  }
}

function documentThemeIcon() {
  if (typeof document === 'undefined') {
    return Moon;
  }
  return document.documentElement.dataset.theme === 'dark' ? Sun : Moon;
}
