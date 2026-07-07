import { Minimize2 } from 'lucide-react';
import type { CSSProperties, PointerEvent, UIEvent } from 'react';
import { useCallback, useEffect, useRef, useState } from 'react';

import { ScrollArea } from '../../components/ui/scroll-area';
import styles from './DiagnosticsWaterfall.module.css';
import controlStyles from './DiagnosticsWaterfallControls.module.css';
import { ResizeHandle } from './DiagnosticsWaterfallGridResizeHandle';
import { WaterfallLeftRow } from './DiagnosticsWaterfallGridRows';
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
  const [hoveredRowId, setHoveredRowId] = useState<string>();
  const [leftHeaderElement, setLeftHeaderElement] = useState<HTMLDivElement | null>(null);
  const [leftPaneElement, setLeftPaneElement] = useState<HTMLDivElement | null>(null);
  const [bottomScrollbarElement, setBottomScrollbarElement] = useState<HTMLDivElement | null>(null);
  const leftHeaderRef = useRef<HTMLDivElement>(null);
  const leftPaneRef = useRef<HTMLDivElement>(null);
  const bottomScrollbarRef = useRef<HTMLDivElement>(null);
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
  const syncFromLeftPane = (event: UIEvent<HTMLDivElement>) => {
    syncHorizontalScroll(event.currentTarget, bottomScrollbarRef.current, leftHeaderRef.current);
  };
  const syncFromBottomScrollbar = (event: UIEvent<HTMLDivElement>) => {
    syncHorizontalScroll(event.currentTarget, leftPaneRef.current, leftHeaderRef.current);
  };
  const clearHoveredRow = (rowId: string) => {
    setHoveredRowId((current) => (current === rowId ? undefined : current));
  };
  const setLeftHeaderRef = useCallback((node: HTMLDivElement | null) => {
    leftHeaderRef.current = node;
    setLeftHeaderElement(node);
  }, []);
  const setLeftPaneRef = useCallback((node: HTMLDivElement | null) => {
    leftPaneRef.current = node;
    setLeftPaneElement(node);
  }, []);
  const setBottomScrollbarRef = useCallback((node: HTMLDivElement | null) => {
    bottomScrollbarRef.current = node;
    setBottomScrollbarElement(node);
  }, []);
  useEffect(() => {
    const leftPane = leftPaneElement;
    const leftHeader = leftHeaderElement;
    const bottomScrollbar = bottomScrollbarElement;
    if (!leftPane || !leftHeader || !bottomScrollbar) {
      return;
    }
    const syncFromLeft = () => syncHorizontalScroll(leftPane, bottomScrollbar, leftHeader);
    const syncFromBottom = () => syncHorizontalScroll(bottomScrollbar, leftPane, leftHeader);
    leftPane.addEventListener('scroll', syncFromLeft, { passive: true });
    bottomScrollbar.addEventListener('scroll', syncFromBottom, { passive: true });
    syncFromLeft();
    return () => {
      leftPane.removeEventListener('scroll', syncFromLeft);
      bottomScrollbar.removeEventListener('scroll', syncFromBottom);
    };
  }, [bottomScrollbarElement, leftHeaderElement, leftPaneElement]);
  return (
    <div className={styles.waterfallGrid} style={style}>
      <ScrollArea
        aria-label="Waterfall rows"
        className={styles.waterfallVerticalScrollArea}
        scrollbars="vertical"
        type="always"
        viewportClassName={styles.waterfallVerticalViewport}
      >
        <div className={styles.waterfallGridViewport}>
          <section aria-label="Waterfall metadata columns" className={styles.waterfallLeftPane}>
            <div className={styles.waterfallLeftHeaderViewport} ref={setLeftHeaderRef}>
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
            </div>
            <div
              className={styles.waterfallLeftContent}
              onScroll={syncFromLeftPane}
              ref={setLeftPaneRef}
            >
              <div className={styles.waterfallLeftRows}>
                {rows.map(({ level, node }) => (
                  <WaterfallLeftRow
                    active={node.selected ?? false}
                    expandedIds={expandedIds}
                    hovered={hoveredRowId === node.id}
                    key={node.id}
                    level={level}
                    node={node}
                    onHoverEnd={() => clearHoveredRow(node.id)}
                    onHoverStart={() => setHoveredRowId(node.id)}
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
                  className={controlStyles.collapseButton}
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
                  data-hovered={hoveredRowId === node.id ? true : undefined}
                  data-waterfall-row-id={node.id}
                  key={node.id}
                  onPointerEnter={() => setHoveredRowId(node.id)}
                  onPointerLeave={() => clearHoveredRow(node.id)}
                >
                  {node.actions}
                </div>
              ))}
            </div>
          </div>
        </div>
      </ScrollArea>
      <ScrollArea
        aria-label="Waterfall metadata horizontal scroll"
        className={styles.waterfallLeftScrollbar}
        onViewportScroll={syncFromBottomScrollbar}
        scrollbars="horizontal"
        type="always"
        viewportClassName={styles.waterfallLeftScrollbarViewport}
        viewportRef={setBottomScrollbarRef}
      >
        <div className={styles.waterfallLeftScrollbarContent} />
      </ScrollArea>
      <div aria-hidden="true" className={styles.waterfallScrollbarSpacer} />
    </div>
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

function syncHorizontalScroll(source: HTMLElement, ...targets: Array<HTMLElement | null>) {
  for (const target of targets) {
    if (target && target.scrollLeft !== source.scrollLeft) {
      target.scrollLeft = source.scrollLeft;
    }
  }
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
