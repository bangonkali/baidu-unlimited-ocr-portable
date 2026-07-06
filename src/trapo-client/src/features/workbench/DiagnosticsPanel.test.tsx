import { describe, expect, test } from 'bun:test';

import type { DiagnosticWorkUnitRecord, IngestRunRecord } from '../../api/types';
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
});
