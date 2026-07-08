import { useEffect, useMemo, useState } from 'react';

import type {
  DiagnosticPipelineTaskRecord,
  DiagnosticWorkUnitRecord,
  DocumentSummary,
  IngestPreviewResultRecord,
  IngestRunRecord,
} from '../../api/types';
import type { TreeNode } from '../../components/workbench';
import { TreeView } from '../../components/workbench';
import styles from './ExplorerTree.module.css';
import { buildDocumentTree } from './ExplorerTreeData';
import type { WorkbenchExplorerFilter } from './workbenchExplorerFilter';
import { latestRunIdFromRuns } from './workbenchExplorerFilter';

interface ExplorerTreeProps {
  documents: DocumentSummary[];
  filter: WorkbenchExplorerFilter;
  rootPath?: string;
  runs: IngestRunRecord[];
  diagnosticWorkUnits?: DiagnosticWorkUnitRecord[];
  previewResults: IngestPreviewResultRecord[];
  selectedFileHash?: string;
  selectedPageNo?: number;
  selectedRunEngineId?: string;
  selectedRunId?: string;
  onFilterChange: (filter: WorkbenchExplorerFilter) => void;
  onSelectDocument: (
    fileHash: string,
    pageNo?: number,
    runId?: string,
    runEngineId?: string,
  ) => void;
  pipelineTasks?: DiagnosticPipelineTaskRecord[];
}

export { buildDocumentTree };

export function ExplorerTree({
  documents,
  diagnosticWorkUnits,
  filter,
  onFilterChange,
  onSelectDocument,
  pipelineTasks,
  previewResults,
  rootPath,
  runs,
  selectedFileHash,
  selectedPageNo,
  selectedRunEngineId,
  selectedRunId,
}: ExplorerTreeProps) {
  const tree = useMemo(
    () =>
      buildDocumentTree({
        documents,
        diagnosticWorkUnits,
        fallbackRootPath: rootPath,
        onSelectDocument,
        pipelineTasks,
        previewResults,
        runId: filter.runId,
        runs,
        scope: filter.scope,
        selectedFileHash,
        selectedPageNo,
        selectedRunEngineId,
        selectedRunId,
      }),
    [
      documents,
      diagnosticWorkUnits,
      filter,
      onSelectDocument,
      pipelineTasks,
      previewResults,
      rootPath,
      runs,
      selectedFileHash,
      selectedPageNo,
      selectedRunEngineId,
      selectedRunId,
    ],
  );
  const [expandedIds, setExpandedIds] = useState(() => defaultExpandedIds(tree.nodes));

  useEffect(() => {
    setExpandedIds((current) => new Set([...current, ...defaultExpandedIds(tree.nodes)]));
  }, [tree.nodes]);

  return (
    <section className={styles.explorer} aria-label="Explorer">
      <div className={styles.header}>
        <span>Explorer</span>
        <RunFilterSelect filter={filter} runs={runs} onChange={onFilterChange} />
      </div>
      <div className={styles.treeScroll}>
        {tree.documentCount === 0 ? <div className={styles.empty}>No documents</div> : null}
        <TreeView
          className={styles.tree}
          expandedIds={expandedIds}
          nodes={tree.nodes}
          onToggle={(id) => toggleExpanded(id, setExpandedIds)}
        />
      </div>
    </section>
  );
}

function RunFilterSelect({
  filter,
  onChange,
  runs,
}: {
  filter: WorkbenchExplorerFilter;
  runs: IngestRunRecord[];
  onChange: (filter: WorkbenchExplorerFilter) => void;
}) {
  const latestRunId = latestRunIdFromRuns(runs);
  const selectedValue = selectedFilterValue(filter, latestRunId);
  const previousRuns = runs.filter((run) => run.run_id !== latestRunId);
  return (
    <select
      aria-label="Explorer run filter"
      className={styles.runSelect}
      disabled={runs.length === 0}
      onChange={(event) => onChange(filterFromValue(event.currentTarget.value))}
      value={selectedValue}
    >
      {latestRunId ? (
        <option value={`run:${latestRunId}`}>Latest run</option>
      ) : (
        <option value="none">No runs</option>
      )}
      <option value="all">All runs</option>
      {previousRuns.map((run) => (
        <option key={run.run_id} value={`run:${run.run_id}`}>
          {runOptionLabel(run)}
        </option>
      ))}
      {filter.scope === 'run' &&
      filter.runId &&
      !runs.some((run) => run.run_id === filter.runId) ? (
        <option value={`run:${filter.runId}`}>{filter.runId}</option>
      ) : null}
    </select>
  );
}

function selectedFilterValue(filter: WorkbenchExplorerFilter, latestRunId: string | undefined) {
  if (filter.scope === 'all') {
    return 'all';
  }
  const runId = filter.runId ?? latestRunId;
  return runId ? `run:${runId}` : 'none';
}

function filterFromValue(value: string): WorkbenchExplorerFilter {
  if (value === 'all') {
    return { scope: 'all' };
  }
  if (value.startsWith('run:')) {
    return { runId: value.slice(4), scope: 'run' };
  }
  return { scope: 'run' };
}

function runOptionLabel(run: IngestRunRecord) {
  const root = rootName(run.root_path);
  return `${root} - ${shortRunId(run.run_id)}`;
}

function rootName(rootPath: string) {
  return (
    rootPath
      .split(/[\\/]+/)
      .filter(Boolean)
      .at(-1) ?? rootPath
  );
}

function shortRunId(runId: string) {
  return runId.length > 10 ? runId.slice(0, 10) : runId;
}

function defaultExpandedIds(nodes: TreeNode[]) {
  const ids = new Set<string>();
  const visit = (node: TreeNode, level: number) => {
    if ((level < 2 || hasSelectedDescendant(node)) && (node.children?.length ?? 0) > 0) {
      ids.add(node.id);
    }
    node.children?.forEach((child) => {
      visit(child, level + 1);
    });
  };
  nodes.forEach((node) => {
    visit(node, 0);
  });
  return ids;
}

function hasSelectedDescendant(node: TreeNode): boolean {
  return (node.children ?? []).some((child) => child.selected || hasSelectedDescendant(child));
}

function toggleExpanded(
  id: string,
  setExpandedIds: (update: (current: Set<string>) => Set<string>) => void,
) {
  setExpandedIds((current) => {
    const next = new Set(current);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    return next;
  });
}
