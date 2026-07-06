import { Minimize2 } from 'lucide-react';
import { useMemo, useState } from 'react';

import {
  useDiagnosticAnalytics,
  useDiagnosticModels,
  useDiagnosticProgress,
  useDiagnosticRuns,
  useDiagnosticTrace,
} from '../../api/hooks';
import type { DiagnosticRunRecord, IngestRunRecord, LogRecord } from '../../api/types';
import { TreeGrid } from '../../components/workbench';
import type { DiagnosticsRouteSearch } from '../../routeSearch';
import { buildProgressNodes, filterLogs, filterRuns, toggled } from './DiagnosticsPanel.helpers';
import styles from './DiagnosticsPanel.module.css';
import {
  AnalyticsView,
  DiagnosticsToolbar,
  LogList,
  ModelLeaseList,
  ProgressSummary,
  TabBar,
} from './DiagnosticsPanel.views';
import { buildWaterfallRunNodes } from './DiagnosticsWaterfallTree';

interface DiagnosticsPanelProps {
  activeRunId?: string | null;
  logs: LogRecord[];
  runs: IngestRunRecord[];
  search?: DiagnosticsRouteSearch;
  onResumeRun?: (runId: string) => void;
  onRestartRun?: (run: IngestRunRecord) => void;
  onSearchChange?: (patch: Partial<DiagnosticsRouteSearch>) => void;
  onStopRun?: (runId?: string) => void;
}

export { filterLogs, filterRuns };

export function DiagnosticsPanel({
  activeRunId,
  logs,
  onResumeRun,
  onRestartRun,
  onSearchChange,
  onStopRun,
  runs,
  search,
}: DiagnosticsPanelProps) {
  const tab = search?.tab ?? 'waterfall';
  const diagnosticRuns = useDiagnosticRuns();
  const selectedRun = search?.run;
  const runRecords = useMemo(
    () => (runs.length > 0 ? runs : (diagnosticRuns.data?.runs.map(diagnosticRunRecord) ?? [])),
    [diagnosticRuns.data?.runs, runs],
  );
  const trace = useDiagnosticTrace({
    limit: 8000,
    q: search?.q,
    run_id: selectedRun,
    status: search?.status === 'all' ? undefined : search?.status,
  });
  const progress = useDiagnosticProgress(selectedRun);
  const analytics = useDiagnosticAnalytics(selectedRun);
  const models = useDiagnosticModels(selectedRun);
  const [expandedIds, setExpandedIds] = useState(() => new Set<string>());
  const waterfallNodes = useMemo(
    () =>
      buildWaterfallRunNodes({
        activeRunId,
        events: trace.data?.events ?? [],
        onResumeRun,
        onRestartRun,
        onStopRun,
        runs: filterRuns(runRecords, search),
        spans: trace.data?.spans ?? [],
        workUnits: progress.data?.work_units ?? [],
      }),
    [
      activeRunId,
      onResumeRun,
      onRestartRun,
      onStopRun,
      progress.data?.work_units,
      runRecords,
      search,
      trace.data?.events,
      trace.data?.spans,
    ],
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
          <div className={styles.waterfallStack}>
            <div className={styles.waterfallControls}>
              {expandedIds.size > 0 ? (
                <button
                  aria-label="Collapse all"
                  className={styles.collapseButton}
                  onClick={() => setExpandedIds(new Set())}
                  title="Collapse all"
                  type="button"
                >
                  <Minimize2 size={13} />
                </button>
              ) : null}
            </div>
            <TreeGrid
              className={styles.tree}
              expandedIds={expandedIds}
              nodes={waterfallNodes}
              onToggle={(id) => setExpandedIds((current) => toggled(current, id))}
            />
          </div>
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

function diagnosticRunRecord(run: DiagnosticRunRecord) {
  return {
    file_hashes: [],
    processed_pages: run.page_count,
    progress_percent: 0,
    queued_files: run.file_count,
    root_path: run.root_path,
    run_id: run.run_id,
    status: run.status,
    total_pages: run.page_count,
  } satisfies IngestRunRecord;
}
