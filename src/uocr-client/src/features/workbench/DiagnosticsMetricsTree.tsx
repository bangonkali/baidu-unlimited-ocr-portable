import type { OcrMetricsTreeNode } from '../../api/types';
import type { TreeDataGridColumn } from '../../components/TreeDataGrid';
import { TreeDataGrid } from '../../components/TreeDataGrid';
import styles from './DiagnosticsPanel.module.css';

const metricColumns: Array<TreeDataGridColumn<OcrMetricsTreeNode>> = [
  {
    header: 'Name',
    id: 'name',
    render: (node) => (
      <span className={styles.metricName}>
        <span className={styles.kindBadge}>{node.kind}</span>
        <span>{node.label}</span>
      </span>
    ),
    width: 'minmax(240px, 1.6fr)',
  },
  {
    header: 'Status',
    id: 'status',
    render: (node) => <strong data-status={node.status}>{node.status}</strong>,
    width: '112px',
  },
  {
    header: 'Model',
    id: 'model',
    render: (node) => node.model_id ?? '',
    width: 'minmax(150px, 1fr)',
  },
  {
    header: 'Runtime',
    id: 'runtime',
    render: (node) => node.runtime_platform || node.runtime_id || '',
    width: 'minmax(150px, 1fr)',
  },
  {
    align: 'right',
    header: 'Tokens',
    id: 'tokens',
    render: (node) => integerLabel(node.token_count),
    width: '84px',
  },
  {
    align: 'right',
    header: 'Avg tok/s',
    id: 'avg',
    render: (node) => tpsLabel(node.avg_tps),
    width: '88px',
  },
  {
    align: 'right',
    header: 'Min',
    id: 'min',
    render: (node) => tpsLabel(node.min_tps),
    width: '70px',
  },
  {
    align: 'right',
    header: 'Max',
    id: 'max',
    render: (node) => tpsLabel(node.max_tps),
    width: '70px',
  },
  {
    align: 'right',
    header: 'Duration',
    id: 'duration',
    render: (node) => durationLabel(node.generation_duration_ms),
    width: '82px',
  },
];

export function MetricsTree({ nodes }: { nodes: OcrMetricsTreeNode[] }) {
  return (
    <TreeDataGrid
      ariaLabel="OCR metrics"
      columns={metricColumns}
      defaultExpandedDepth={2}
      emptyLabel="No OCR metrics"
      nodes={nodes}
    />
  );
}

function integerLabel(value: number | undefined) {
  return Math.round(value ?? 0).toLocaleString();
}

function tpsLabel(value: number | undefined) {
  return value && value > 0 ? value.toFixed(1) : '';
}

function durationLabel(ms: number | undefined) {
  if (!ms) {
    return '';
  }
  if (ms < 1000) {
    return `${ms} ms`;
  }
  return `${(ms / 1000).toFixed(1)} s`;
}
