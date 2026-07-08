import type {
  DocumentSummary,
  IngestEngineConfigRecord,
  IngestPreviewResultRecord,
  IngestRunRecord,
} from '../../api/types';

interface EngineResultOptionsInput {
  document?: DocumentSummary;
  results: IngestPreviewResultRecord[];
  run?: IngestRunRecord;
}

interface SelectedRunEngineInput {
  explicitResultId?: string;
  results: IngestPreviewResultRecord[];
  selectionRunEngineId?: string;
}

export function engineResultOptions({
  document,
  results,
  run,
}: EngineResultOptionsInput): IngestPreviewResultRecord[] {
  const configs = run?.engine_configs ?? [];
  if (configs.length === 0) {
    return sortByOrdinal(results);
  }

  const resultsByEngine = new Map(results.map((result) => [result.run_engine_id, result]));
  const configIds = new Set(configs.map((config) => config.run_engine_id));
  const merged = configs.map((config) => {
    const result = resultsByEngine.get(config.run_engine_id);
    return result ? resultWithLiveStatus(result, config) : resultFromEngineConfig(config, document);
  });

  for (const result of results) {
    if (!configIds.has(result.run_engine_id)) {
      merged.push(result);
    }
  }

  return sortByOrdinal(merged);
}

export function selectedRunEngineIdFromOptions({
  explicitResultId,
  results,
  selectionRunEngineId,
}: SelectedRunEngineInput) {
  if (explicitResultId) {
    return explicitResultId;
  }
  if (
    selectionRunEngineId &&
    results.some((result) => result.run_engine_id === selectionRunEngineId)
  ) {
    return selectionRunEngineId;
  }
  return defaultRunEngineId(results);
}

export function defaultRunEngineId(results: IngestPreviewResultRecord[]) {
  return runningResult(results)?.run_engine_id ?? latestRanResult(results)?.run_engine_id;
}

export function previewableEngineResults(results: IngestPreviewResultRecord[]) {
  return results.filter((result) => result.status === 'running' || hasPreviewOutput(result));
}

function runningResult(results: IngestPreviewResultRecord[]) {
  return [...results]
    .filter((result) => result.status === 'running')
    .sort((left, right) => right.ordinal - left.ordinal)[0];
}

function latestRanResult(results: IngestPreviewResultRecord[]) {
  return [...results]
    .filter((result) => hasPreviewOutput(result) || isRanStatus(result.status))
    .sort((left, right) => right.ordinal - left.ordinal)[0];
}

function hasPreviewOutput(result: IngestPreviewResultRecord) {
  return result.output_count > 0 || result.page_count > 0;
}

function isRanStatus(status: string) {
  return status === 'completed' || status === 'completed_with_errors' || status === 'failed';
}

function resultWithLiveStatus(
  result: IngestPreviewResultRecord,
  config: IngestEngineConfigRecord,
): IngestPreviewResultRecord {
  return {
    ...result,
    error: config.error ?? result.error,
    status: config.status === 'running' ? config.status : result.status,
  };
}

function resultFromEngineConfig(
  config: IngestEngineConfigRecord,
  document?: DocumentSummary,
): IngestPreviewResultRecord {
  const pageCount =
    config.status === 'running' || config.usable_output_count > 0 ? (document?.page_count ?? 0) : 0;
  return {
    engine_id: config.engine_id,
    engine_kind: config.engine_kind,
    error: config.error,
    label: config.label,
    model_id: config.model_id,
    ordinal: config.ordinal,
    output_count: config.usable_output_count,
    page_count: pageCount,
    previewer: config.previewer,
    profile_id: config.profile_id,
    provenance: { source: 'engine_config' },
    run_engine_id: config.run_engine_id,
    run_id: config.run_id,
    runner_kind: config.engine_kind,
    runner_status: config.status === 'completed' ? 'ready' : config.status,
    runtime_id: config.runtime_id,
    status: config.status,
  };
}

function sortByOrdinal(results: IngestPreviewResultRecord[]) {
  return [...results].sort(
    (left, right) => left.ordinal - right.ordinal || left.label.localeCompare(right.label),
  );
}
