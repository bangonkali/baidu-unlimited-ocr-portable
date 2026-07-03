import { useQuery } from '@tanstack/react-query';

import { buildApiUrl, getJson } from './http';
import { queryKeys } from './queryKeys';
import type {
  DiagnosticAnalyticsPayload,
  DiagnosticModelsPayload,
  DiagnosticProgressPayload,
  DiagnosticRunsPayload,
  DiagnosticTracePayload,
  OcrReplayPayload,
} from './types';

export function useOcrReplay(params: {
  run_id?: string;
  file_hash?: string;
  page_no?: number;
  since_sequence?: number;
  limit?: number;
  enabled?: boolean;
  refetchInterval?: number | false;
}) {
  const { enabled, refetchInterval, ...query } = params;
  return useQuery({
    enabled: (enabled ?? true) && Boolean(query.run_id || query.file_hash),
    placeholderData: { events: [] },
    queryFn: ({ signal }) =>
      getJson<OcrReplayPayload>(buildApiUrl('/api/ocr/events', query), signal),
    queryKey: queryKeys.ocrReplay(query),
    refetchInterval,
  });
}

export function useDiagnosticRuns(limit = 100) {
  return useQuery({
    placeholderData: { runs: [] },
    queryFn: ({ signal }) =>
      getJson<DiagnosticRunsPayload>(buildApiUrl('/api/diagnostics/runs', { limit }), signal),
    queryKey: queryKeys.diagnosticRuns,
  });
}

export function useDiagnosticTrace(params: {
  run_id?: string;
  file_hash?: string;
  page_no?: number;
  status?: string;
  q?: string;
  limit?: number;
}) {
  return useQuery({
    placeholderData: {
      events: [],
      spans: [],
      summary: { error_count: 0, event_count: 0, span_count: 0, total_duration_ms: 0 },
    },
    queryFn: ({ signal }) =>
      getJson<DiagnosticTracePayload>(buildApiUrl('/api/diagnostics/trace', params), signal),
    queryKey: queryKeys.diagnosticTrace(params),
  });
}

export function useDiagnosticProgress(runId?: string, limit = 5000) {
  return useQuery({
    placeholderData: {
      model_leases: [],
      summary: {
        cancelled: 0,
        completed: 0,
        failed: 0,
        queued: 0,
        running: 0,
        total_work_units: 0,
      },
      work_units: [],
    },
    queryFn: ({ signal }) =>
      getJson<DiagnosticProgressPayload>(
        buildApiUrl('/api/diagnostics/progress', { limit, run_id: runId }),
        signal,
      ),
    queryKey: queryKeys.diagnosticProgress(runId),
  });
}

export function useDiagnosticAnalytics(runId?: string, limit = 10000) {
  return useQuery({
    queryFn: ({ signal }) =>
      getJson<DiagnosticAnalyticsPayload>(
        buildApiUrl('/api/diagnostics/analytics', { limit, run_id: runId }),
        signal,
      ),
    queryKey: queryKeys.diagnosticAnalytics(runId),
  });
}

export function useDiagnosticModels(runId?: string, limit = 1000) {
  return useQuery({
    placeholderData: { model_leases: [] },
    queryFn: ({ signal }) =>
      getJson<DiagnosticModelsPayload>(
        buildApiUrl('/api/diagnostics/models', { limit, run_id: runId }),
        signal,
      ),
    queryKey: queryKeys.diagnosticModels(runId),
  });
}
