import { useMemo, useState } from 'react';

import {
  useDiagnosticAnalytics,
  useDiagnosticModels,
  useDiagnosticProgress,
  useDiagnosticRuns,
  useDiagnosticTrace,
} from '../../api/hooks';
import type { IngestRunRecord, LogRecord } from '../../api/types';
import { TreeGrid } from '../../components/workbench';
import type { DiagnosticsRouteSearch } from '../../routeSearch';
import {
  buildProgressNodes,
  buildSpanNodes,
  filterLogs,
  filterRuns,
  toggled,
} from './DiagnosticsPanel.helpers';
import styles from './DiagnosticsPanel.module.css';
import {
  AnalyticsView,
  DiagnosticsToolbar,
  LogList,
  ModelLeaseList,
  ProgressSummary,
  TabBar,
} from './DiagnosticsPanel.views';

interface DiagnosticsPanelProps {
  logs: LogRecord[];
  runs: IngestRunRecord[];
  search?: DiagnosticsRouteSearch;
  onSearchChange?: (patch: Partial<DiagnosticsRouteSearch>) => void;
}

export { filterLogs, filterRuns };

export function DiagnosticsPanel({ logs, onSearchChange, search }: DiagnosticsPanelProps) {
  const tab = search?.tab ?? 'waterfall';
  const diagnosticRuns = useDiagnosticRuns();
  const selectedRun = search?.run ?? diagnosticRuns.data?.runs[0]?.run_id;
  const trace = useDiagnosticTrace({
    limit: 8000,
    q: search?.q,
    run_id: selectedRun,
    status: search?.status === 'all' ? undefined : search?.status,
  });
  const progress = useDiagnosticProgress(selectedRun);
  const analytics = useDiagnosticAnalytics(selectedRun);
  const models = useDiagnosticModels(selectedRun);
  const [expandedIds, setExpandedIds] = useState(() => new Set<string>(['root']));
  const waterfallNodes = useMemo(
    () => buildSpanNodes(trace.data?.spans ?? []),
    [trace.data?.spans],
  );
  const progressNodes = useMemo(
    () => buildProgressNodes(progress.data?.work_units ?? []),
    [progress.data?.work_units],
  );

  return (
    <section className={styles.panel} aria-label="Diagnostics" data-tour="diagnostics">
      <div className={styles.header}>Diagnostics</div>
      <TabBar active={tab} onChange={(nextTab) => onSearchChange?.({ tab: nextTab })} />
      <DiagnosticsToolbar
        query={search?.q ?? ''}
        runId={selectedRun}
        runs={diagnosticRuns.data?.runs.map((run) => run.run_id) ?? []}
        status={search?.status ?? 'all'}
        onChange={onSearchChange}
      />
      <div className={styles.body}>
        {tab === 'waterfall' ? (
          <TreeGrid
            className={styles.tree}
            expandedIds={expandedIds}
            nodes={waterfallNodes}
            onToggle={(id) => setExpandedIds((current) => toggled(current, id))}
          />
        ) : null}
        {tab === 'progress' ? (
          <ProgressSummary summary={progress.data?.summary} nodes={progressNodes} />
        ) : null}
        {tab === 'analytics' ? <AnalyticsView data={analytics.data} /> : null}
        {tab === 'models' ? <ModelLeaseList leases={models.data?.model_leases ?? []} /> : null}
        {tab === 'logs' ? <LogList logs={filterLogs(logs, search)} /> : null}
      </div>
    </section>
  );
}
