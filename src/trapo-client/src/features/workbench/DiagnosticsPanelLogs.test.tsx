import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { renderToString } from 'react-dom/server';

import type { DiagnosticWaterfallRowRecord } from '../../api/types';
import { fixtureLogs, fixtureRuns } from '../../stories/fixtures/workbenchFixtures';
import { filterLogs, filterRuns } from './DiagnosticsPanel';
import { buildFailureSummary } from './DiagnosticsPanel.helpers';
import { DiagnosticsToolbar, FailureSummary, LogList } from './DiagnosticsPanel.views';

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
});

describe('diagnostics log rendering', () => {
  test('renders logs with timestamps and bulk copy action', () => {
    const html = renderToString(<LogList logs={fixtureLogs} />);
    expect(html).toContain('Copy All');
    expect(html).toContain('Time');
    expect(html).toContain('Component');
    expect(html).toContain(fixtureLogs[0]?.message ?? '');
  });

  test('renders log level and component filter controls', () => {
    const html = renderToString(
      <DiagnosticsToolbar
        component="native-stderr"
        components={['native-stderr', 'server']}
        level="ERROR"
        levels={['ERROR', 'WARN']}
        query=""
        runId={undefined}
        runs={[]}
        status="all"
        onChange={() => undefined}
      />,
    );

    expect(html).toContain('aria-label="Log level"');
    expect(html).toContain('All levels');
    expect(html).toContain('aria-label="Log component"');
    expect(html).toContain('native-stderr');
  });

  test('keeps error log level available even when recent rows are warnings', () => {
    const html = renderToString(
      <DiagnosticsToolbar
        component=""
        components={['native-stderr']}
        level=""
        levels={['ERROR', 'WARN', 'INFO']}
        query=""
        runId={undefined}
        runs={[]}
        status="all"
        onChange={() => undefined}
      />,
    );

    expect(html).toContain('<option value="ERROR">ERROR</option>');
  });

  test('renders a compact failure summary from failed diagnostic rows', () => {
    const failures = buildFailureSummary(
      [
        diagnosticRow({
          error_message:
            'PaddleOCR-VL 1.6 create engine failed: llama.cpp cuda backend was not compiled into this build',
          label: 'PaddleOCR-VL 1.6',
          pipeline_step: 'ingest.engine',
          status: 'failed',
        }),
      ],
      [],
    );
    const html = renderToString(<FailureSummary items={failures} />);

    expect(html).toContain('Diagnostic failures');
    expect(html).toContain('PaddleOCR-VL 1.6 create engine failed');
  });

  test('marks warning and error log rows for visible styling', () => {
    const html = renderToString(
      <LogList
        logs={[
          {
            component: 'ocr',
            level: 'ERROR',
            message: 'PP-OCRv6 create engine failed',
            timestamp: '2026-07-09T00:00:00Z',
          },
          {
            component: 'native-stderr',
            level: 'WARN',
            message: 'llama.cpp warning',
            timestamp: '2026-07-09T00:00:01Z',
          },
        ]}
      />,
    );

    expect(html).toContain('data-level="ERROR"');
    expect(html).toContain('data-level="WARN"');
    expect(html).toContain('PP-OCRv6 create engine failed');
  });

  test('styles diagnostic errors as visible detail blocks', () => {
    const css = readFileSync(
      new URL('./DiagnosticWorkUnitDetail.module.css', import.meta.url),
      'utf8',
    );
    const logsCss = readFileSync(new URL('./DiagnosticsLogs.module.css', import.meta.url), 'utf8');

    expect(css).toContain('.detailRow[data-tone="error"]');
    expect(css).toContain('.detailRowExtra');
    expect(logsCss).toContain('.logRow[data-level="ERROR"]');
    expect(logsCss).toContain('.logRow[data-level="WARN"]');
  });
});

function diagnosticRow(patch: Partial<DiagnosticWaterfallRowRecord>): DiagnosticWaterfallRowRecord {
  return {
    activity_kind: 'internal',
    attributes: {},
    category: 'engine',
    child_count: 0,
    depth: 0,
    duration_ms: 1,
    label: 'diagnostic',
    pipeline_step: 'ingest.engine',
    row_source: 'diagnostic_span',
    sort_index: 0,
    span_id: 'span-1',
    span_kind: 'engine',
    status: 'ok',
    status_code: 'ok',
    visual_duration_ms: 1,
    ...patch,
  };
}
