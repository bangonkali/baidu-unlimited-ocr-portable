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
  extra?: string;
  id: string;
  label: string;
  meta: string;
  tone?: 'error';
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
        <div className={styles.detailRow} data-tone={row.tone} key={row.id}>
          <span>{row.label}</span>
          <small>{row.detail}</small>
          <strong>{row.meta}</strong>
          {row.extra ? <pre className={styles.detailRowExtra}>{row.extra}</pre> : null}
        </div>
      ))}
    </section>
  );
}

function spanRow(span: DiagnosticSpanRecord): DetailRow {
  const errorDetail = span.error_message ?? span.status_message ?? span.error_stack ?? undefined;
  return {
    detail: errorDetail ?? `${span.pipeline_step} · ${span.activity_kind}/${span.span_kind}`,
    extra: errorDetail,
    id: span.span_id,
    label: span.name,
    meta:
      span.status_code && span.status_code !== 'unset'
        ? `${span.status_code} · ${formatMs(span.duration_ms)}`
        : formatMs(span.duration_ms),
    tone:
      errorDetail || span.status === 'failed' || span.status_code === 'error' ? 'error' : undefined,
  };
}

function eventRow(event: DiagnosticEventRecord): DetailRow {
  const attributes = nonEmptyJson(event.attributes);
  return {
    detail: event.message,
    extra: attributes,
    id: event.event_id,
    label: event.name,
    meta: event.timestamp_ms
      ? `${event.severity} · ${formatEventTime(event.timestamp_ms)}`
      : event.severity,
    tone: event.severity === 'ERROR' ? 'error' : undefined,
  };
}

function nonEmptyJson(value: Record<string, unknown>) {
  return Object.keys(value).length === 0 ? undefined : JSON.stringify(value, null, 2);
}

function formatEventTime(timestampMs: number) {
  const date = new Date(timestampMs);
  return `${String(date.getUTCHours()).padStart(2, '0')}:${String(date.getUTCMinutes()).padStart(
    2,
    '0',
  )}:${String(date.getUTCSeconds()).padStart(2, '0')}`;
}
