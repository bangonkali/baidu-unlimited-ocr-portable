import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { renderToString } from 'react-dom/server';

import { fixtureLogs, fixtureRuns } from '../../stories/fixtures/workbenchFixtures';
import { filterLogs, filterRuns } from './DiagnosticsPanel';
import { DiagnosticsToolbar, LogList } from './DiagnosticsPanel.views';

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
