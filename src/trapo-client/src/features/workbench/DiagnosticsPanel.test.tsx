import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { renderToString } from 'react-dom/server';

import type { DiagnosticWaterfallPayload, DiagnosticWaterfallRowRecord } from '../../api/types';
import type { TreeGridNode } from '../../components/workbench';
import { fixtureLogs, fixtureRuns } from '../../stories/fixtures/workbenchFixtures';
import { filterLogs, filterRuns } from './DiagnosticsPanel';
import { formatMs } from './DiagnosticsPanel.helpers';
import { LogList } from './DiagnosticsPanel.views';
import { clampWaterfallColumnWidth, DiagnosticsWaterfallGrid } from './DiagnosticsWaterfallGrid';
import { buildWaterfallRunNodes } from './DiagnosticsWaterfallTree';

describe('diagnostics filters', () => {
  test('filters logs by query, level, and component', () => {
    expect(filterLogs(fixtureLogs, { component: 'pdfium', level: 'INFO' })).toHaveLength(1);
    expect(filterLogs(fixtureLogs, { q: 'pdfium' })[0]?.component).toBe('pdfium');
    expect(filterLogs(fixtureLogs, { level: 'ERROR' })).toHaveLength(0);
  });

  test('filters runs by status, run id, and query', () => {
    expect(filterRuns(fixtureRuns, { status: 'running' })).toHaveLength(1);
    expect(filterRuns(fixtureRuns, { status: 'all' })).toHaveLength(fixtureRuns.length);
    expect(filterRuns(fixtureRuns, { run: 'run-20260629-01' })).toHaveLength(1);
    expect(filterRuns(fixtureRuns, { q: 'missing' })).toHaveLength(0);
  });

  test('renders logs with timestamps and bulk copy action', () => {
    const html = renderToString(<LogList logs={fixtureLogs} />);

    expect(html).toContain('Copy All');
    expect(html).toContain('Time');
    expect(html).toContain('Component');
    expect(html).toContain(fixtureLogs[0]?.message ?? '');
  });
});

describe('diagnostics waterfall', () => {
  test('groups waterfall rows by trace parent id', () => {
    const nodes = buildWaterfallRunNodes({
      payload: waterfallPayload([
        waterfallRow({
          child_count: 1,
          label: 'Run run-a',
          row_id: 'run:run-a',
          row_source: 'run',
          span_id: null,
          span_kind: 'run',
        }),
        waterfallRow({
          child_count: 1,
          label: 'generate_embedding - completed',
          parent_row_id: 'run:run-a',
          row_source: 'pipeline_task',
          row_id: 'task:task-a',
          sort_index: 1,
          span_kind: 'task',
          task_id: 'task-a',
        }),
        waterfallRow({
          file_hash: 'file-a',
          label: 'Generate page embeddings',
          page_no: 1,
          parent_row_id: 'task:task-a',
          row_id: 'span:page-a',
          sort_index: 2,
          span_id: 'page-a',
          span_kind: 'embedding_page',
        }),
      ]),
    });

    expect(nodes).toHaveLength(1);
    expect(nodes[0]?.label).toBe('Run run-a');
    expect(nodes[0]?.actions).toBeDefined();
    expect(nodes[0]?.children?.[0]?.label).toBe('Task: generate_embedding - completed');
    expect(nodes[0]?.children?.[0]?.children?.[0]?.label).toBe(
      'Generate page embeddings - file-a - p1',
    );
  });
});

describe('diagnostics waterfall layout', () => {
  test('renders waterfall grid with resizable metadata columns before the waterfall', () => {
    const nodes = buildWaterfallRunNodes({
      payload: waterfallPayload([
        waterfallRow({
          label: 'Run run-a',
          row_source: 'run',
          start_ms: Date.UTC(2026, 6, 7, 1, 2, 3, 4),
          visual_start_ms: Date.UTC(2026, 6, 7, 1, 2, 3, 4),
        }),
      ]),
    });
    const html = renderToString(
      <DiagnosticsWaterfallGrid
        expandedIds={new Set()}
        nodes={nodes}
        onCollapseAll={() => undefined}
        onToggle={() => undefined}
      />,
    );
    expect(html.indexOf('Name')).toBeLessThan(html.indexOf('Timestamp'));
    expect(html.indexOf('Timestamp')).toBeLessThan(html.indexOf('Timespan'));
    expect(html).toContain('2026-07-07 01:02:03.004');
    expect(html).toContain('1.00s');
    expect(html).toContain('Resize name column');
    expect(html).toContain('Resize timestamp column');
    expect(html).toContain('Resize timespan column');
    expect(html).toContain('Resize waterfall column');
    expect(html).toContain('Waterfall</span>');
    expect(html).toContain('Waterfall rows');
    expect(html).toContain('Waterfall metadata horizontal scroll');
    expect(html.match(/data-waterfall-row-id="span:operation"/g)).toHaveLength(2);
    expect(html.match(/Waterfall metadata columns/g)).toHaveLength(1);
  });

  test('keeps waterfall rows aligned with a pinned metadata scrollbar', () => {
    const panelCss = readFileSync(
      new URL('./DiagnosticsPanel.module.css', import.meta.url),
      'utf8',
    );
    const css = readFileSync(new URL('./DiagnosticsWaterfall.module.css', import.meta.url), 'utf8');

    expect(panelCss).toContain('.body[data-tab="waterfall"]');
    expect(panelCss).toContain('max-height: 100%;');
    expect(css).toContain('--waterfall-header-height: 24px;');
    expect(css).toContain('--waterfall-row-height: 22px;');
    expect(css).toContain('grid-template-rows: minmax(0, 1fr) var(--waterfall-scrollbar-height);');
    expect(css).toContain('.waterfallVerticalScrollArea');
    expect(css).toContain('align-items: start;');
    expect(css).toContain('.waterfallLeftScrollbar');
    expect(css).toContain('height: var(--waterfall-scrollbar-height);');
    expect(css).toContain('.waterfallLeftRow[data-hovered="true"]');
    expect(css).toContain('.waterfallRightRow[data-hovered="true"]');
    expect(css).toContain('position: sticky;');
    expect(css).toContain('height: var(--waterfall-row-height);');
  });

  test('clamps waterfall split and metadata column resizing', () => {
    expect(clampWaterfallColumnWidth('leftPane', 120, 1000)).toBe(320);
    expect(clampWaterfallColumnWidth('leftPane', 900, 1000)).toBe(640);
    expect(clampWaterfallColumnWidth('leftPane', 500, 1000)).toBe(500);
    expect(clampWaterfallColumnWidth('timestamp', 320, 1000)).toBe(280);
    expect(clampWaterfallColumnWidth('timespan', 48, 1000)).toBe(72);
    expect(clampWaterfallColumnWidth('name', 900, 1000)).toBe(820);
  });
});

describe('diagnostics waterfall rendering', () => {
  test('does not duplicate file group labels', () => {
    const nodes = buildWaterfallRunNodes({
      payload: waterfallPayload([
        waterfallRow({
          file_hash: 'file-a',
          filename: 'trace.pdf',
          label: 'trace.pdf',
          row_id: 'file-group:run-a:file-a:ocr',
          row_source: 'file_group',
          span_id: null,
          span_kind: 'file_group',
        }),
      ]),
    });

    expect(nodes[0]?.label).toBe('trace.pdf');
  });

  test('renders waterfall bars with timeline offset and duration lanes', () => {
    const nodes = buildWaterfallRunNodes({
      payload: waterfallPayload([
        waterfallRow({
          duration_ms: 1000,
          end_ms: 1000,
          label: 'first',
          row_id: 'span:first',
          visual_duration_ms: 1000,
          visual_end_ms: 1000,
        }),
        waterfallRow({
          duration_ms: 2000,
          end_ms: 4000,
          label: 'second',
          row_id: 'span:second',
          sort_index: 1,
          start_ms: 2000,
          visual_duration_ms: 2000,
          visual_end_ms: 4000,
          visual_start_ms: 2000,
        }),
      ]),
    });
    const secondNode = findNode(nodes, 'span:second');
    const html = renderToString(secondNode?.actions);

    expect(html).toContain('second 2.00s');
    expect(html).toContain('left:50%');
    expect(html).toContain('width:50%');
  });

  test('styles waterfall bars as full-height row fills', () => {
    const css = readFileSync(
      new URL('./DiagnosticsWaterfallBars.module.css', import.meta.url),
      'utf8',
    );

    expect(css).toContain('--waterfall-bar-fill: var(--accent);');
    expect(css).toContain('height: 100%;');
    expect(css).toContain('top: 0;');
    expect(css).toContain('bottom: 0;');
    expect(css).toContain('inset 0 -1px 0');
    expect(css).not.toContain('height: 10px;');
    expect(css).not.toContain('height: 6px;');
  });

  test('formats nonzero sub-millisecond spans as less than one millisecond', () => {
    expect(formatMs(0.3)).toBe('<1ms');
    expect(formatMs(0)).toBe('0ms');
  });
});

function waterfallPayload(rows: DiagnosticWaterfallRowRecord[]): DiagnosticWaterfallPayload {
  return {
    rows,
    summary: {
      duration_ms: 4000,
      end_ms: 4000,
      error_count: 0,
      row_count: rows.length,
      start_ms: 0,
      trace_count: 1,
    },
  };
}

function waterfallRow(
  overrides: Partial<DiagnosticWaterfallRowRecord> = {},
): DiagnosticWaterfallRowRecord {
  return {
    attributes: {},
    category: 'operation',
    child_count: 0,
    depth: 0,
    duration_ms: 1000,
    end_ms: 1000,
    ended_at: '2026-07-07T00:00:01.000Z',
    error_message: null,
    error_type: null,
    file_hash: null,
    filename: null,
    label: 'operation',
    page_no: null,
    parent_row_id: null,
    pipeline_step: 'test',
    row_source: 'diagnostic_span',
    row_id: 'span:operation',
    run_id: 'run-a',
    sort_index: 0,
    span_id: 'operation',
    span_kind: 'operation',
    start_ms: 0,
    started_at: '2026-07-07T00:00:00.000Z',
    status: 'ok',
    task_id: null,
    trace_id: 'run-a',
    visual_duration_ms: 1000,
    visual_end_ms: 1000,
    visual_start_ms: 0,
    work_unit_id: null,
    ...overrides,
  };
}

function findNode(nodes: TreeGridNode[], id: string): TreeGridNode | undefined {
  for (const node of nodes) {
    if (node.id === id) {
      return node;
    }
    const found = findNode(node.children ?? [], id);
    if (found) {
      return found;
    }
  }
  return undefined;
}
