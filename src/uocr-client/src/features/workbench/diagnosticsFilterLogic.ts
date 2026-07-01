import type { IngestRunRecord, LogRecord, OcrMetricsTreeNode } from '../../api/types';
import type { DiagnosticsRouteSearch } from '../../routeSearch';

export function filterLogs(logs: LogRecord[], search: DiagnosticsRouteSearch | undefined) {
  const query = normalized(search?.q);
  return logs.filter((log) => {
    if (search?.level && log.level !== search.level) {
      return false;
    }
    if (search?.component && log.component !== search.component) {
      return false;
    }
    if (!query) {
      return true;
    }
    return `${log.timestamp} ${log.level} ${log.component} ${log.message}`
      .toLowerCase()
      .includes(query);
  });
}

export function filterRuns(runs: IngestRunRecord[], search: DiagnosticsRouteSearch | undefined) {
  const query = normalized(search?.q);
  return runs.filter((run) => {
    if (search?.run && run.run_id !== search.run) {
      return false;
    }
    if (search?.status && run.status !== search.status) {
      return false;
    }
    if (!query) {
      return true;
    }
    return `${run.run_id} ${run.status} ${run.root_path} ${run.profile_id ?? ''}`
      .toLowerCase()
      .includes(query);
  });
}

export function filterMetricNodes(
  nodes: OcrMetricsTreeNode[],
  search: DiagnosticsRouteSearch | undefined,
): OcrMetricsTreeNode[] {
  const query = normalized(search?.q);
  return nodes
    .map((node) => filterMetricNode(node, search, query))
    .filter((node): node is OcrMetricsTreeNode => Boolean(node));
}

export function flattenMetricNodes(nodes: OcrMetricsTreeNode[]): OcrMetricsTreeNode[] {
  const flat: OcrMetricsTreeNode[] = [];
  const visit = (items: OcrMetricsTreeNode[]) => {
    for (const node of items) {
      flat.push(node);
      if (node.children?.length) {
        visit(node.children);
      }
    }
  };
  visit(nodes);
  return flat;
}

function filterMetricNode(
  node: OcrMetricsTreeNode,
  search: DiagnosticsRouteSearch | undefined,
  query: string,
): OcrMetricsTreeNode | null {
  const children = node.children
    ?.map((child) => filterMetricNode(child, search, query))
    .filter((child): child is OcrMetricsTreeNode => Boolean(child));
  const matchesRun = !search?.run || node.run_id === search.run;
  const matchesStatus = !search?.status || node.status === search.status;
  const haystack =
    `${node.label} ${node.run_id} ${node.file_hash ?? ''} ${node.model_id ?? ''} ${node.runtime_id ?? ''} ${
      node.runtime_platform ?? ''
    } ${node.accelerator ?? ''}`.toLowerCase();
  const matchesQuery = !query || haystack.includes(query);
  if ((matchesRun && matchesStatus && matchesQuery) || (children?.length ?? 0) > 0) {
    return { ...node, children };
  }
  return null;
}

function normalized(value: string | undefined) {
  return value?.trim().toLowerCase() ?? '';
}
