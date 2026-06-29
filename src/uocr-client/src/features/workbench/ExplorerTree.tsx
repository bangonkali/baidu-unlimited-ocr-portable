import { getCoreRowModel, useReactTable } from '@tanstack/react-table';
import { useVirtualizer } from '@tanstack/react-virtual';
import { FileText } from 'lucide-react';
import { useMemo, useRef } from 'react';

import type { DocumentSummary } from '../../api/types';
import { setSelection } from '../../stores/workbenchStore';
import styles from './ExplorerTree.module.css';

interface ExplorerTreeProps {
  documents: DocumentSummary[];
  selectedFileHash?: string;
}

export function ExplorerTree({ documents, selectedFileHash }: ExplorerTreeProps) {
  const parentRef = useRef<HTMLDivElement>(null);
  const selectedFiles = useMemo(
    () => new Set(selectedFileHash ? [selectedFileHash] : []),
    [selectedFileHash],
  );
  const columns = useMemo(
    () => [
      {
        accessorKey: 'display_name',
        cell: (info: { getValue: () => unknown }) => String(info.getValue()),
        header: 'Document',
      },
    ],
    [],
  );
  const table = useReactTable({
    columns,
    data: documents,
    getCoreRowModel: getCoreRowModel(),
  });
  const rows = table.getRowModel().rows;
  const virtualizer = useVirtualizer({
    count: rows.length,
    estimateSize: () => 32,
    getScrollElement: () => parentRef.current,
    overscan: 8,
  });

  return (
    <section className={styles.explorer} aria-label="Explorer">
      <div className={styles.header}>Explorer</div>
      <div className={styles.scroll} ref={parentRef}>
        <div className={styles.virtualSpace} style={{ height: virtualizer.getTotalSize() }}>
          {rows.length === 0 ? <div className={styles.empty}>No documents</div> : null}
          <DocumentRows rows={rows} selectedFiles={selectedFiles} virtualizer={virtualizer} />
        </div>
      </div>
    </section>
  );
}

function DocumentRows(props: {
  rows: ReturnType<ReturnType<typeof useReactTable<DocumentSummary>>['getRowModel']>['rows'];
  selectedFiles: Set<string>;
  virtualizer: ReturnType<typeof useVirtualizer<HTMLDivElement, Element>>;
}) {
  return props.virtualizer.getVirtualItems().map((virtualRow) => {
    const row = props.rows[virtualRow.index];
    if (!row) {
      return null;
    }
    const document = row.original;
    return (
      <DocumentRow
        document={document}
        isActive={props.selectedFiles.has(document.file_hash)}
        key={row.id}
        offset={virtualRow.start}
      />
    );
  });
}

function DocumentRow(props: { document: DocumentSummary; isActive: boolean; offset: number }) {
  const { document } = props;
  return (
    <button
      className={styles.row}
      data-active={props.isActive}
      onClick={() => setSelection({ fileHash: document.file_hash, pageNo: 1 })}
      style={{ transform: `translateY(${props.offset}px)` }}
      type="button"
    >
      <FileText size={15} />
      <span>{document.display_name}</span>
      <span className={styles.badge}>{document.status}</span>
    </button>
  );
}
