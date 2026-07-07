import { Activity, Database, FileText, Server } from 'lucide-react';
import type { ReactNode } from 'react';

import { useDiagnosticWorkUnitDetail } from '../../api/hooks';
import type { DiagnosticEventRecord, DiagnosticSpanRecord } from '../../api/types';
import { formatMs, iconForStatus } from './DiagnosticsPanel.helpers';
import styles from './DiagnosticWorkUnitDetail.module.css';

export function DiagnosticWorkUnitDetail({ workUnitId }: { workUnitId?: string }) {
  const detail = useDiagnosticWorkUnitDetail(workUnitId);
  if (!workUnitId) {
    return null;
  }
  if (detail.isLoading) {
    return <div className={styles.detailEmpty}>Loading work unit...</div>;
  }
  if (detail.isError || !detail.data) {
    return <div className={styles.detailEmpty}>Work unit not found.</div>;
  }
  const unit = detail.data.work_unit;
  return (
    <section className={styles.detailPanel} aria-label="Work unit detail">
      <div className={styles.detailHeader}>
        {iconForStatus(unit.status)}
        <strong>{unit.phase}</strong>
        <span>{unit.work_unit_id}</span>
      </div>
      <div className={styles.detailMetrics}>
        <span>{unit.status}</span>
        <span>{unit.engine}</span>
        <span>{unit.model || 'no model'}</span>
        <span>{unit.page_no ? `page ${unit.page_no}` : 'document'}</span>
      </div>
      <div className={styles.detailBody}>
        <DetailSection
          icon={<Activity size={14} />}
          title="Spans"
          empty="No spans recorded"
          rows={detail.data.spans.map(spanRow)}
        />
        <DetailSection
          icon={<FileText size={14} />}
          title="Events"
          empty="No events recorded"
          rows={detail.data.events.map(eventRow)}
        />
        <DetailSection
          icon={<Server size={14} />}
          title="Model Leases"
          empty="No model leases recorded"
          rows={detail.data.model_leases.map((lease) => ({
            detail: lease.provider,
            id: lease.lease_id,
            label: lease.model,
            meta: lease.duration_ms ? formatMs(lease.duration_ms) : lease.status,
          }))}
        />
        <div className={styles.detailJsonBlock}>
          <div>
            <Database size={14} />
            <strong>Metadata</strong>
          </div>
          <pre>{JSON.stringify(unit.metadata, null, 2)}</pre>
        </div>
      </div>
    </section>
  );
}

interface DetailRow {
  detail: string;
  id: string;
  label: string;
  meta: string;
}

function DetailSection({
  empty,
  icon,
  rows,
  title,
}: {
  empty: string;
  icon: ReactNode;
  rows: DetailRow[];
  title: string;
}) {
  return (
    <section className={styles.detailSection}>
      <div className={styles.detailSectionHeader}>
        {icon}
        <strong>{title}</strong>
      </div>
      {rows.length === 0 ? <div className={styles.detailEmpty}>{empty}</div> : null}
      {rows.map((row) => (
        <div className={styles.detailRow} key={row.id}>
          <span>{row.label}</span>
          <small>{row.detail}</small>
          <strong>{row.meta}</strong>
        </div>
      ))}
    </section>
  );
}

function spanRow(span: DiagnosticSpanRecord): DetailRow {
  return {
    detail: span.pipeline_step,
    id: span.span_id,
    label: span.name,
    meta: formatMs(span.duration_ms),
  };
}

function eventRow(event: DiagnosticEventRecord): DetailRow {
  return {
    detail: event.message,
    id: event.event_id,
    label: event.name,
    meta: event.severity,
  };
}
