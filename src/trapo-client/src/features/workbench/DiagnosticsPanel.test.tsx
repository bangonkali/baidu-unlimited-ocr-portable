import { describe, expect, test } from 'bun:test';
import { renderToString } from 'react-dom/server';

import type { DiagnosticWorkUnitRecord, IngestRunRecord } from '../../api/types';
import type { TreeGridNode } from '../../components/workbench';
import { fixtureLogs, fixtureRuns } from '../../stories/fixtures/workbenchFixtures';
import { filterLogs, filterRuns } from './DiagnosticsPanel';
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

  test('groups waterfall records under collapsed run roots', () => {
    const run = {
      can_resume: true,
      root_path: '/data/incoming',
      run_id: 'run-a',
      status: 'cancelled',
    } satisfies IngestRunRecord;
    const workUnit = {
      artifact_variant: null,
      attempt_count: 1,
      engine: 'pdfium',
      error: null,
      execution_key: 'file-a:1:ocr',
      file_hash: 'file-a',
      filename: 'invoice.pdf',
      finished_at: null,
      metadata: { relative_path: 'invoices/2026/invoice.pdf' },
      model: 'model-a',
      page_no: 1,
      phase: 'ocr',
      profile: 'default',
      provider: 'local',
      result: {},
      run_id: 'run-a',
      source_path: null,
      started_at: null,
      status: 'completed',
      work_key: 'file-a:1:ocr',
      work_unit_id: 'work-a',
    } satisfies DiagnosticWorkUnitRecord;

    const nodes = buildWaterfallRunNodes({
      events: [],
      pipelineTasks: [],
      runs: [run],
      spans: [],
      workUnits: [workUnit],
    });

    expect(nodes).toHaveLength(1);
    expect(nodes[0]?.actions).toBeDefined();
    expect(nodes[0]?.children?.[0]?.label).toBe('invoices');
    expect(nodes[0]?.children?.[0]?.children?.[0]?.label).toBe('2026');
    expect(nodes[0]?.children?.[0]?.children?.[0]?.children?.[0]?.label).toBe('invoice.pdf');
    expect(nodes[0]?.children?.[0]?.children?.[0]?.children?.[0]?.children?.[0]?.label).toBe(
      'page 1',
    );
  });

  test('renders waterfall bars with timeline offset and duration lanes', () => {
    const run = {
      root_path: '/data/incoming',
      run_id: 'run-a',
      status: 'completed',
    } satisfies IngestRunRecord;
    const first = workUnit({
      finished_at: '2026-07-07T00:00:01.000Z',
      started_at: '2026-07-07T00:00:00.000Z',
      work_unit_id: 'work-first',
    });
    const second = workUnit({
      finished_at: '2026-07-07T00:00:04.000Z',
      started_at: '2026-07-07T00:00:02.000Z',
      work_unit_id: 'work-second',
    });

    const nodes = buildWaterfallRunNodes({
      events: [],
      pipelineTasks: [],
      runs: [run],
      spans: [],
      workUnits: [first, second],
    });
    const secondNode = findNode(nodes, 'work:work-second');
    const html = renderToString(secondNode?.actions);

    expect(html).toContain('title="2.00s"');
    expect(html).toContain('left:50%');
    expect(html).toContain('width:50%');
    expect(html).toContain('2.00s');
  });
});

function workUnit(overrides: Partial<DiagnosticWorkUnitRecord> = {}): DiagnosticWorkUnitRecord {
  return {
    artifact_variant: null,
    attempt_count: 1,
    engine: 'pdfium',
    error: null,
    execution_key: 'file-a:1:ocr',
    file_hash: 'file-a',
    filename: 'invoice.pdf',
    finished_at: null,
    metadata: { relative_path: 'invoice.pdf' },
    model: 'model-a',
    page_no: 1,
    phase: 'ocr',
    profile: 'default',
    provider: 'local',
    result: {},
    run_id: 'run-a',
    source_path: null,
    started_at: null,
    status: 'completed',
    work_key: 'file-a:1:ocr',
    work_unit_id: 'work-a',
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
