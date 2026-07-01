import type { ColumnDef } from '@tanstack/react-table';
import { flexRender, getCoreRowModel, useReactTable } from '@tanstack/react-table';
import { ArrowDownAZ, ArrowUpAZ } from 'lucide-react';
import { useMemo } from 'react';

import type { ModelAssetRecord } from '../../api/types';
import type { ModelSortKey, SortDirection } from '../../routeSearch';
import type { ModelActionHandlers } from './ModelActions';
import { ModelActions } from './ModelActions';
import styles from './ModelDataGrid.module.css';
import { FilesCell, ModelCell, ProgressCell } from './ModelGridCells';
import { formatBytes } from './modelDownloadFormat';
import { modelRequiredBytes } from './modelLibrary';
import { statusIcon } from './modelStatus';

interface ModelDataGridProps extends ModelActionHandlers {
  busy?: boolean;
  dir: SortDirection;
  models: ModelAssetRecord[];
  sort: ModelSortKey;
  onSortChange: (sort: ModelSortKey) => void;
}

export function ModelDataGrid({
  busy,
  dir,
  models,
  onCancelModel,
  onDownloadModel,
  onSelectModel,
  onSortChange,
  sort,
}: ModelDataGridProps) {
  const columns = useMemo(
    () =>
      modelColumns({
        busy,
        onCancelModel,
        onDownloadModel,
        onSelectModel,
      }),
    [busy, onCancelModel, onDownloadModel, onSelectModel],
  );
  const table = useReactTable({
    columns,
    data: models,
    getCoreRowModel: getCoreRowModel(),
  });
  return (
    <div className={styles.gridViewport}>
      <table className={styles.modelGrid}>
        <thead>
          {table.getHeaderGroups().map((headerGroup) => (
            <tr key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <th key={header.id} scope="col">
                  {sortKeyForColumn(header.column.id) ? (
                    <button
                      className={styles.sortHeader}
                      onClick={() => onSortChange(sortKeyForColumn(header.column.id) ?? 'status')}
                      type="button"
                    >
                      {flexRender(header.column.columnDef.header, header.getContext())}
                      {sort === sortKeyForColumn(header.column.id) ? sortIcon(dir) : null}
                    </button>
                  ) : (
                    flexRender(header.column.columnDef.header, header.getContext())
                  )}
                </th>
              ))}
            </tr>
          ))}
        </thead>
        <tbody>
          {table.getRowModel().rows.map((row) => (
            <tr data-selected={row.original.selected === true} key={row.id}>
              {row.getVisibleCells().map((cell) => (
                <td key={cell.id}>{flexRender(cell.column.columnDef.cell, cell.getContext())}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      {models.length === 0 ? <div className={styles.empty}>No models match this view.</div> : null}
    </div>
  );
}

function modelColumns(handlers: ModelActionHandlers & { busy?: boolean }) {
  return [
    {
      accessorKey: 'display_name',
      header: 'Model',
      cell: ({ row }) => <ModelCell model={row.original} />,
    },
    {
      accessorKey: 'status',
      header: 'Status',
      cell: ({ row }) => (
        <span className={styles.statusBadge} data-status={row.original.status}>
          {statusIcon(row.original.status)}
          {row.original.status}
        </span>
      ),
    },
    {
      accessorKey: 'bits',
      header: 'Bits',
      cell: ({ row }) => (row.original.bits ? `${row.original.bits}-bit` : 'mixed'),
    },
    {
      accessorKey: 'hardware_tier',
      header: 'VRAM / Tier',
      cell: ({ row }) => row.original.hardware_tier ?? 'Runtime default',
    },
    {
      id: 'size',
      header: 'Size',
      cell: ({ row }) => formatBytes(modelRequiredBytes(row.original)),
    },
    {
      id: 'progress',
      header: 'Progress',
      cell: ({ row }) => <ProgressCell model={row.original} />,
    },
    {
      id: 'files',
      header: 'Files',
      cell: ({ row }) => <FilesCell model={row.original} />,
    },
    {
      id: 'actions',
      header: 'Actions',
      cell: ({ row }) => <ModelActions compact model={row.original} {...handlers} />,
    },
  ] satisfies Array<ColumnDef<ModelAssetRecord>>;
}

function sortKeyForColumn(columnId: string): ModelSortKey | undefined {
  switch (columnId) {
    case 'display_name':
      return 'name';
    case 'status':
      return 'status';
    case 'bits':
      return 'bits';
    case 'hardware_tier':
      return 'vram';
    case 'size':
      return 'size';
    case 'progress':
      return 'progress';
    default:
      return undefined;
  }
}

function sortIcon(dir: SortDirection) {
  const Icon = dir === 'desc' ? ArrowDownAZ : ArrowUpAZ;
  return <Icon size={12} strokeWidth={1.9} />;
}
