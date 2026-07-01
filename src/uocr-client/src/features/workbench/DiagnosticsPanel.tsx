import type { IngestRunRecord, LogRecord, OcrMetricsTreePayload } from '../../api/types';
import type { DiagnosticsRouteSearch } from '../../routeSearch';
import { DiagnosticsFilters } from './DiagnosticsFilters';
import { LogList, RunList } from './DiagnosticsLists';
import { MetricsTree } from './DiagnosticsMetricsTree';
import styles from './DiagnosticsPanel.module.css';
import {
  filterLogs,
  filterMetricNodes,
  filterRuns,
  flattenMetricNodes,
} from './diagnosticsFilterLogic';

interface DiagnosticsPanelProps {
  logs: LogRecord[];
  metrics: OcrMetricsTreePayload;
  runs: IngestRunRecord[];
  search?: DiagnosticsRouteSearch;
  onSearchChange?: (patch: Partial<DiagnosticsRouteSearch>) => void;
}

export function DiagnosticsPanel({
  logs,
  metrics,
  onSearchChange,
  runs,
  search,
}: DiagnosticsPanelProps) {
  const tab = search?.tab ?? 'runs';
  const filteredLogs = filterLogs(logs, search);
  const filteredMetrics = filterMetricNodes(metrics.nodes, search);
  const filteredRuns = filterRuns(runs, search);
  const metricRows = flattenMetricNodes(metrics.nodes);

  return (
    <section className={styles.panel} aria-label="Diagnostics" data-tour="diagnostics">
      <div className={styles.header}>Diagnostics</div>
      <div className={styles.tabs}>
        <button
          className={styles.tab}
          data-active={tab === 'runs'}
          onClick={() => onSearchChange?.({ tab: 'runs' })}
          type="button"
        >
          Runs
        </button>
        <button
          className={styles.tab}
          data-active={tab === 'metrics'}
          onClick={() => onSearchChange?.({ tab: 'metrics' })}
          type="button"
        >
          Metrics
        </button>
        <button
          className={styles.tab}
          data-active={tab === 'logs'}
          onClick={() => onSearchChange?.({ tab: 'logs' })}
          type="button"
        >
          Logs
        </button>
      </div>
      <DiagnosticsFilters
        logs={logs}
        metrics={metricRows}
        runs={runs}
        search={search}
        tab={tab}
        onSearchChange={onSearchChange}
      />
      <div className={styles.body}>
        {tab === 'runs' ? <RunList runs={filteredRuns} /> : null}
        {tab === 'metrics' ? <MetricsTree nodes={filteredMetrics} /> : null}
        {tab === 'logs' ? <LogList logs={filteredLogs} /> : null}
      </div>
    </section>
  );
}
