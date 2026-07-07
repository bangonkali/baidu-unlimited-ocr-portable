import { ChevronDown, ChevronRight } from 'lucide-react';
import type { CSSProperties, ReactNode } from 'react';

import styles from './DiagnosticsWaterfall.module.css';
import type { DiagnosticWaterfallNode } from './DiagnosticsWaterfallTree';

interface WaterfallLeftRowProps {
  active: boolean;
  expandedIds: Set<string>;
  hovered: boolean;
  level: number;
  node: DiagnosticWaterfallNode;
  onHoverEnd: () => void;
  onHoverStart: () => void;
  onToggle: (id: string) => void;
}

export function WaterfallLeftRow({
  active,
  expandedIds,
  hovered,
  level,
  node,
  onHoverEnd,
  onHoverStart,
  onToggle,
}: WaterfallLeftRowProps) {
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
      data-hovered={hovered ? true : undefined}
      data-waterfall-row-id={node.id}
      id={node.id}
      onPointerEnter={onHoverStart}
      onPointerLeave={onHoverEnd}
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
