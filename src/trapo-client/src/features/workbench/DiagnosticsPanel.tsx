import { useNavigate } from '@tanstack/react-router';
import { useCallback, useEffect, useMemo, useState } from 'react';

import {
  useDiagnosticAnalytics,
  useDiagnosticModels,
  useDiagnosticProgress,
  useDiagnosticRuns,
  useDiagnosticWaterfall,
} from '../../api/hooks';
import type { IngestRunRecord, LogRecord } from '../../api/types';
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
import { DiagnosticsWaterfallGrid } from './DiagnosticsWaterfallGrid';
import { buildWaterfallRunNodes, waterfallExpandableIds } from './DiagnosticsWaterfallTree';
import { DiagnosticWorkUnitDetail } from './DiagnosticWorkUnitDetail';

interface DiagnosticsPanelProps {
  activeRunId?: string | null;
  logs: LogRecord[];
  runs: IngestRunRecord[];
  search?: DiagnosticsRouteSearch;
  workUnitId?: string;
  onResumeRun?: (runId: string) => void;
  onRestartRun?: (run: IngestRunRecord) => void;
  onSearchChange?: (patch: Partial<DiagnosticsRouteSearch>) => void;
  onStopRun?: (runId?: string) => void;
}

export { filterLogs, filterRuns };

export function DiagnosticsPanel({
  logs,
  onSearchChange,
  search,
  workUnitId,
}: DiagnosticsPanelProps) {
  const navigate = useNavigate();
  const tab = search?.tab ?? 'waterfall';
  const openWorkUnit = useCallback(
    (nextWorkUnitId: string) => {
      void navigate({
        params: { workUnitId: nextWorkUnitId },
        search: (current) => ({ ...current, ...(search ?? {}) }),
        to: '/diagnostics/$workUnitId',
      });
    },
    [navigate, search],
  );
  const diagnosticRuns = useDiagnosticRuns();
  const selectedRun = search?.run;
  const waterfall = useDiagnosticWaterfall({
    limit: 8000,
    q: search?.q,
    refetchInterval: 1500,
    run_id: selectedRun,
    status: search?.status === 'all' ? undefined : search?.status,
  });
  const progress = useDiagnosticProgress(selectedRun, 5000, 1500);
  const analytics = useDiagnosticAnalytics(selectedRun);
  const models = useDiagnosticModels(selectedRun);
  const [expandedIds, setExpandedIds] = useState(() => new Set<string>());
  const waterfallNodes = useMemo(
    () =>
      buildWaterfallRunNodes({
        onWorkUnitSelect: openWorkUnit,
        payload: waterfall.data,
      }),
    [openWorkUnit, waterfall.data],
  );
  const hasEmbeddingWaterfallRows =
    waterfall.data?.rows.some((row) => row.pipeline_step === 'generate_embedding') ?? true;
  useEffect(() => {
    setExpandedIds((current) =>
      current.size === 0 ? waterfallExpandableIds(waterfallNodes) : current,
    );
  }, [waterfallNodes]);
  const progressNodes = useMemo(
    () => buildProgressNodes(progress.data?.work_units ?? [], openWorkUnit),
    [openWorkUnit, progress.data?.work_units],
  );
  const logLevels = useMemo(() => uniqueLogValues(logs, 'level'), [logs]);
  const logComponents = useMemo(() => uniqueLogValues(logs, 'component'), [logs]);

  return (
    <section className={styles.panel} aria-label="Diagnostics" data-tour="diagnostics">
      <div className={styles.header}>Diagnostics</div>
      <TabBar active={tab} onChange={(nextTab) => onSearchChange?.({ tab: nextTab })} />
      <DiagnosticsToolbar
        query={search?.q ?? ''}
        component={search?.component ?? ''}
        components={logComponents}
        level={search?.level ?? ''}
        levels={logLevels}
        runId={selectedRun}
        runs={diagnosticRuns.data?.runs.map((run) => run.run_id) ?? []}
        status={search?.status ?? 'all'}
        onChange={onSearchChange}
      />
      <div className={styles.body} data-tab={tab}>
        {tab === 'waterfall' ? (
          <div className={styles.waterfallStack}>
            {!hasEmbeddingWaterfallRows ? (
              <div className={styles.waterfallNotice}>
                No embedding generation recorded for this run.
              </div>
            ) : null}
            <DiagnosticsWaterfallGrid
              expandedIds={expandedIds}
              nodes={waterfallNodes}
              onCollapseAll={() => setExpandedIds(new Set())}
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
      {workUnitId ? <DiagnosticWorkUnitDetail workUnitId={workUnitId} /> : null}
    </section>
  );
}

function uniqueLogValues(logs: LogRecord[], key: 'component' | 'level') {
  return [...new Set(logs.map((log) => log[key]).filter(Boolean))].sort((left, right) =>
    left.localeCompare(right),
  );
}
