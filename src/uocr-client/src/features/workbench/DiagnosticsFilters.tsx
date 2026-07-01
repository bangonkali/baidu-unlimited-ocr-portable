import type { IngestRunRecord, LogRecord, OcrMetricsTreeNode } from '../../api/types';
import type { DiagnosticsRouteSearch } from '../../routeSearch';
import styles from './DiagnosticsPanel.module.css';

interface DiagnosticsFiltersProps {
  logs: LogRecord[];
  metrics: OcrMetricsTreeNode[];
  runs: IngestRunRecord[];
  search?: DiagnosticsRouteSearch;
  tab: 'logs' | 'metrics' | 'runs';
  onSearchChange?: (patch: Partial<DiagnosticsRouteSearch>) => void;
}

export function DiagnosticsFilters({
  logs,
  metrics,
  onSearchChange,
  runs,
  search,
  tab,
}: DiagnosticsFiltersProps) {
  return (
    <div className={styles.filters}>
      <input
        aria-label="Filter diagnostics"
        onChange={(event) => onSearchChange?.({ q: event.target.value || undefined })}
        placeholder="Filter diagnostics"
        value={search?.q ?? ''}
      />
      {tab === 'logs' ? (
        <>
          <FilterSelect
            label="Level"
            onChange={(level) => onSearchChange?.({ level })}
            options={uniqueValues(logs.map((log) => log.level))}
            value={search?.level}
          />
          <FilterSelect
            label="Component"
            onChange={(component) => onSearchChange?.({ component })}
            options={uniqueValues(logs.map((log) => log.component))}
            value={search?.component}
          />
        </>
      ) : tab === 'metrics' ? (
        <>
          <FilterSelect
            label="Status"
            onChange={(status) => onSearchChange?.({ status })}
            options={uniqueValues(metrics.map((item) => item.status))}
            value={search?.status}
          />
          <FilterSelect
            label="Run"
            onChange={(run) => onSearchChange?.({ run })}
            options={uniqueValues(metrics.map((item) => item.run_id))}
            value={search?.run}
          />
        </>
      ) : (
        <>
          <FilterSelect
            label="Status"
            onChange={(status) => onSearchChange?.({ status })}
            options={uniqueValues(runs.map((run) => String(run.status)))}
            value={search?.status}
          />
          <FilterSelect
            label="Run"
            onChange={(run) => onSearchChange?.({ run })}
            options={uniqueValues(runs.map((item) => item.run_id))}
            value={search?.run}
          />
        </>
      )}
    </div>
  );
}

function FilterSelect({
  label,
  onChange,
  options,
  value,
}: {
  label: string;
  options: string[];
  value?: string;
  onChange: (value: string | undefined) => void;
}) {
  return (
    <label>
      <span className={styles.filterLabelText}>{label}</span>
      <select onChange={(event) => onChange(event.target.value || undefined)} value={value ?? ''}>
        <option value="">All</option>
        {options.map((option) => (
          <option key={option} value={option}>
            {option}
          </option>
        ))}
      </select>
    </label>
  );
}

function uniqueValues(values: string[]) {
  return [...new Set(values.filter(Boolean))].sort((left, right) => left.localeCompare(right));
}
