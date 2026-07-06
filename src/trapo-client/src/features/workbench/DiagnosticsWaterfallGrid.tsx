import { ChevronDown, ChevronRight, Minimize2 } from 'lucide-react';
import type { CSSProperties, PointerEvent, ReactNode } from 'react';
import { useState } from 'react';

import styles from './DiagnosticsWaterfall.module.css';
import type { DiagnosticWaterfallNode } from './DiagnosticsWaterfallTree';

interface DiagnosticsWaterfallGridProps {
  expandedIds: Set<string>;
  nodes: DiagnosticWaterfallNode[];
  onCollapseAll: () => void;
  onToggle: (id: string) => void;
}

interface WaterfallColumns {
  name: number;
  timespan: number;
  timestamp: number;
}

interface VisibleWaterfallNode {
  level: number;
  node: DiagnosticWaterfallNode;
}

export function DiagnosticsWaterfallGrid({
  expandedIds,
  nodes,
  onCollapseAll,
  onToggle,
}: DiagnosticsWaterfallGridProps) {
  const [columns, setColumns] = useState<WaterfallColumns>({
    name: 420,
    timespan: 92,
    timestamp: 190,
  });
  const rows = visibleWaterfallRows(nodes, expandedIds);
  const style = {
    '--waterfall-name-column': `${columns.name}px`,
    '--waterfall-timespan-column': `${columns.timespan}px`,
    '--waterfall-timestamp-column': `${columns.timestamp}px`,
  } as CSSProperties;
  const beginResize = (column: keyof WaterfallColumns, event: PointerEvent<HTMLButtonElement>) => {
    event.preventDefault();
    const startX = event.clientX;
    const startWidth = columns[column];
    const handlePointerMove = (moveEvent: globalThis.PointerEvent) => {
      const width = clampColumnWidth(column, startWidth + moveEvent.clientX - startX);
      setColumns((current) => ({ ...current, [column]: width }));
    };
    const handlePointerUp = () => {
      window.removeEventListener('pointermove', handlePointerMove);
      window.removeEventListener('pointerup', handlePointerUp);
    };
    window.addEventListener('pointermove', handlePointerMove);
    window.addEventListener('pointerup', handlePointerUp, { once: true });
  };
  return (
    <div className={styles.waterfallGrid} style={style}>
      <div className={styles.waterfallGridViewport}>
        <section aria-label="Waterfall metadata columns" className={styles.waterfallLeftPane}>
          <div className={styles.waterfallLeftContent}>
            <div className={styles.waterfallLeftHeader}>
              <span>Name</span>
              <ResizeHandle
                label="Resize name column"
                onPointerDown={(event) => beginResize('name', event)}
              />
              <span>Timestamp</span>
              <ResizeHandle
                label="Resize timestamp column"
                onPointerDown={(event) => beginResize('timestamp', event)}
              />
              <span>Timespan</span>
              <ResizeHandle
                label="Resize timespan column"
                onPointerDown={(event) => beginResize('timespan', event)}
              />
            </div>
            <div className={styles.waterfallLeftRows}>
              {rows.map(({ level, node }) => (
                <WaterfallLeftRow
                  active={node.selected ?? false}
                  expandedIds={expandedIds}
                  key={node.id}
                  level={level}
                  node={node}
                  onToggle={onToggle}
                />
              ))}
            </div>
          </div>
        </section>
        <div className={styles.waterfallRightPane}>
          <div className={styles.waterfallGridActionHeader}>
            Waterfall
            {expandedIds.size > 0 ? (
              <button
                aria-label="Collapse all"
                className={styles.waterfallCollapseButton}
                onClick={onCollapseAll}
                title="Collapse all"
                type="button"
              >
                <Minimize2 size={13} />
              </button>
            ) : null}
          </div>
          <div className={styles.waterfallRightRows}>
            {rows.map(({ node }) => (
              <div
                className={styles.waterfallRightRow}
                data-active={node.selected ? true : undefined}
                key={node.id}
              >
                {node.actions}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

function WaterfallLeftRow({
  active,
  expandedIds,
  level,
  node,
  onToggle,
}: {
  active: boolean;
  expandedIds: Set<string>;
  level: number;
  node: DiagnosticWaterfallNode;
  onToggle: (id: string) => void;
}) {
  const childCount = node.children?.length ?? 0;
  const hasChildren = node.hasChildren ?? childCount > 0;
  const expanded = expandedIds.has(node.id);
  const toggle = () => {
    if (!expanded && childCount === 0) {
      node.onExpand?.();
    }
    onToggle(node.id);
  };
  return (
    <div
      className={styles.waterfallLeftRow}
      data-active={active ? true : undefined}
      id={node.id}
      style={{ '--tree-level': String(level) } as CSSProperties}
    >
      <span className={styles.waterfallNameCell}>
        <span className={styles.waterfallNameContent}>
          <WaterfallTwisty
            expanded={expanded}
            hasChildren={hasChildren}
            label={node.label}
            onToggle={toggle}
          />
          <WaterfallNameButton icon={node.icon} label={node.label} onSelect={node.onSelect} />
        </span>
      </span>
      <span aria-hidden="true" className={styles.waterfallRowDivider} />
      <span className={styles.waterfallTimestampCell}>{node.timestamp}</span>
      <span aria-hidden="true" className={styles.waterfallRowDivider} />
      <span className={styles.waterfallTimespanCell}>{node.timespan}</span>
      <span aria-hidden="true" className={styles.waterfallRowDivider} />
    </div>
  );
}

function ResizeHandle({
  label,
  onPointerDown,
}: {
  label: string;
  onPointerDown: (event: PointerEvent<HTMLButtonElement>) => void;
}) {
  return (
    <button
      aria-label={label}
      className={styles.waterfallResizeHandle}
      onPointerDown={onPointerDown}
      title={label}
      type="button"
    />
  );
}

function WaterfallTwisty({
  expanded,
  hasChildren,
  label,
  onToggle,
}: {
  expanded: boolean;
  hasChildren: boolean;
  label: string;
  onToggle: () => void;
}) {
  return (
    <button
      aria-label={expanded ? `Collapse ${label}` : `Expand ${label}`}
      className={styles.waterfallTwisty}
      disabled={!hasChildren}
      onClick={onToggle}
      type="button"
    >
      {hasChildren ? expanded ? <ChevronDown size={13} /> : <ChevronRight size={13} /> : null}
    </button>
  );
}

function WaterfallNameButton({
  icon,
  label,
  onSelect,
}: {
  icon?: ReactNode;
  label: string;
  onSelect?: () => void;
}) {
  return (
    <button className={styles.waterfallNameButton} onClick={onSelect} type="button">
      {icon}
      <span>{label}</span>
    </button>
  );
}

function clampColumnWidth(column: keyof WaterfallColumns, width: number) {
  if (column === 'timestamp') {
    return clamp(width, 150, 280);
  }
  if (column === 'timespan') {
    return clamp(width, 72, 160);
  }
  return clamp(width, 180, 820);
}

function visibleWaterfallRows(nodes: DiagnosticWaterfallNode[], expandedIds: Set<string>) {
  const rows: VisibleWaterfallNode[] = [];
  const visit = (node: DiagnosticWaterfallNode, level: number) => {
    rows.push({ level, node });
    if (!expandedIds.has(node.id)) {
      return;
    }
    node.children?.forEach((child) => {
      visit(child, level + 1);
    });
  };
  nodes.forEach((node) => {
    visit(node, 0);
  });
  return rows;
}

function clamp(value: number, minimum: number, maximum: number) {
  return Math.min(Math.max(value, minimum), maximum);
}
