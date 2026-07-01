import type {
  DocumentRegionsPayload,
  DocumentTextPayload,
  OcrMetricsTreeNode,
  OcrMetricsTreePayload,
  PageTextRecord,
  TextRegionSpan,
} from '../api/types';
import type {
  OcrPageMetricsPayload,
  OcrPageRegionRemovePayload,
  OcrPageRegionUpsertPayload,
  OcrPageSpanRemovePayload,
  OcrPageSpanUpsertPayload,
  OcrPageStreamContext,
  OcrPageTextPatchPayload,
} from './realtimeTypes';

export function ensureTextPage(
  current: DocumentTextPayload | undefined,
  context: OcrPageStreamContext,
): DocumentTextPayload {
  const payload = current ?? { file_hash: context.file_hash, pages: [] };
  return {
    ...payload,
    file_hash: context.file_hash,
    pages: upsertPage(payload.pages, { page_no: context.page_no, spans: [], text: '' }),
  };
}

export function applyTextPatch(
  current: DocumentTextPayload | undefined,
  patch: OcrPageTextPatchPayload,
): DocumentTextPayload {
  const payload = ensureTextPage(current, patch);
  return {
    ...payload,
    pages: payload.pages.map((page) =>
      page.page_no === patch.page_no
        ? {
            ...page,
            text:
              patch.op === 'append'
                ? replaceRange(page.text, patch.start, patch.end, patch.text)
                : patch.text,
          }
        : page,
    ),
  };
}

export function applySpanUpsert(
  current: DocumentTextPayload | undefined,
  payload: OcrPageSpanUpsertPayload,
): DocumentTextPayload {
  const textPayload = ensureTextPage(current, payload);
  return {
    ...textPayload,
    pages: textPayload.pages.map((page) =>
      page.page_no === payload.page_no
        ? {
            ...page,
            spans: upsertSpan(page.spans, payload.span),
          }
        : page,
    ),
  };
}

export function applySpanRemove(
  current: DocumentTextPayload | undefined,
  payload: OcrPageSpanRemovePayload,
): DocumentTextPayload {
  const textPayload = ensureTextPage(current, payload);
  return {
    ...textPayload,
    pages: textPayload.pages.map((page) =>
      page.page_no === payload.page_no
        ? { ...page, spans: page.spans.filter((span) => span.region_id !== payload.region_id) }
        : page,
    ),
  };
}

export function applyRegionUpsert(
  current: DocumentRegionsPayload | undefined,
  payload: OcrPageRegionUpsertPayload,
): DocumentRegionsPayload {
  const existing = current ?? { boxes: [], file_hash: payload.file_hash };
  const boxes = existing.boxes.some((box) => box.region_id === payload.region.region_id)
    ? existing.boxes.map((box) =>
        box.region_id === payload.region.region_id ? payload.region : box,
      )
    : [...existing.boxes, payload.region];
  return { boxes, file_hash: payload.file_hash };
}

export function applyRegionRemove(
  current: DocumentRegionsPayload | undefined,
  payload: OcrPageRegionRemovePayload,
): DocumentRegionsPayload {
  const existing = current ?? { boxes: [], file_hash: payload.file_hash };
  return {
    boxes: existing.boxes.filter((box) => box.region_id !== payload.region_id),
    file_hash: payload.file_hash,
  };
}

export function applyMetricPatch(
  current: OcrMetricsTreePayload | undefined,
  payload: OcrPageMetricsPayload,
): OcrMetricsTreePayload {
  const nodes = (current?.nodes ?? []).map(cloneNode);
  const runId = `run:${payload.run_id}`;
  const fileId = `file:${payload.run_id}:${payload.file_hash}`;
  const pageId = `page:${payload.run_id}:${payload.file_hash}:${payload.page_no}`;
  const run =
    nodes.find((node) => node.id === runId) ?? emptyNode(runId, 'run', payload.run_id, payload);
  const runWithoutOld = nodes.filter((node) => node.id !== runId);
  const file =
    run.children?.find((node) => node.id === fileId) ??
    emptyNode(fileId, 'file', payload.file_hash, payload);
  const otherFiles = (run.children ?? []).filter((node) => node.id !== fileId);
  const page = pageMetricNode(pageId, payload);
  const fileChildren = [...(file.children ?? []).filter((node) => node.id !== pageId), page].sort(
    comparePageNodes,
  );
  const nextFile = rollupNode({ ...file, children: fileChildren });
  const nextRun = rollupNode({ ...run, children: [...otherFiles, nextFile].sort(compareLabels) });
  return { nodes: [...runWithoutOld, nextRun].sort(compareLabels) };
}

function upsertPage(pages: PageTextRecord[], page: PageTextRecord) {
  const next = pages.some((item) => item.page_no === page.page_no)
    ? pages
    : [...pages, page].sort((left, right) => left.page_no - right.page_no);
  return next;
}

function replaceRange(value: string, start: number, end: number, text: string) {
  return `${value.slice(0, Math.max(0, start))}${text}${value.slice(Math.max(start, end))}`;
}

function upsertSpan(spans: TextRegionSpan[], span: TextRegionSpan) {
  const next = spans.some((item) => item.region_id === span.region_id)
    ? spans.map((item) => (item.region_id === span.region_id ? span : item))
    : [...spans, span];
  return next.sort((left, right) => left.start - right.start || left.end - right.end);
}

function cloneNode(node: OcrMetricsTreeNode): OcrMetricsTreeNode {
  return { ...node, children: node.children?.map(cloneNode) };
}

function emptyNode(
  id: string,
  kind: OcrMetricsTreeNode['kind'],
  label: string,
  payload: OcrPageMetricsPayload,
): OcrMetricsTreeNode {
  return {
    accelerator: payload.accelerator,
    avg_tps: 0,
    chunk_count: 0,
    children: [],
    elapsed_ms: 0,
    error: null,
    file_hash: kind === 'run' ? undefined : payload.file_hash,
    first_token_latency_ms: 0,
    generation_duration_ms: 0,
    id,
    kind,
    label,
    max_tps: 0,
    min_tps: 0,
    model_id: payload.model_id,
    page_count: 0,
    run_id: payload.run_id,
    runtime_id: payload.runtime_id,
    runtime_platform: payload.runtime_platform,
    status: payload.status,
    token_count: 0,
  };
}

function pageMetricNode(id: string, payload: OcrPageMetricsPayload): OcrMetricsTreeNode {
  return {
    accelerator: payload.accelerator,
    avg_tps: payload.avg_tps,
    chunk_count: payload.chunk_count,
    completed_at: payload.completed_at,
    elapsed_ms: payload.elapsed_ms,
    error: payload.error,
    file_hash: payload.file_hash,
    first_token_latency_ms: payload.first_token_latency_ms,
    generation_duration_ms: payload.generation_duration_ms,
    id,
    kind: 'page',
    label: `Page ${payload.page_no}`,
    max_tps: payload.max_tps,
    min_tps: payload.min_tps,
    model_id: payload.model_id,
    page_count: 1,
    page_no: payload.page_no,
    run_id: payload.run_id,
    runtime_id: payload.runtime_id,
    runtime_platform: payload.runtime_platform,
    started_at: payload.started_at,
    status: payload.status,
    token_count: payload.token_count,
  };
}

function rollupNode(node: OcrMetricsTreeNode): OcrMetricsTreeNode {
  const children = node.children ?? [];
  const tokenCount = sum(children, 'token_count');
  const duration = sum(children, 'generation_duration_ms');
  return {
    ...node,
    accelerator: firstValue(children, 'accelerator') ?? node.accelerator,
    avg_tps: tokenCount > 0 && duration > 0 ? tokenCount / (duration / 1000) : 0,
    chunk_count: sum(children, 'chunk_count'),
    elapsed_ms: sum(children, 'elapsed_ms'),
    error: children.find((child) => child.error)?.error ?? null,
    first_token_latency_ms: minPositive(children.map((child) => child.first_token_latency_ms)),
    generation_duration_ms: duration,
    max_tps: Math.max(0, ...children.map((child) => child.max_tps)),
    min_tps: minPositive(children.map((child) => child.min_tps)),
    model_id: firstValue(children, 'model_id') ?? node.model_id,
    page_count: sum(children, 'page_count'),
    runtime_id: firstValue(children, 'runtime_id') ?? node.runtime_id,
    runtime_platform: firstValue(children, 'runtime_platform') ?? node.runtime_platform,
    started_at: minText(children.map((child) => child.started_at)),
    completed_at: maxText(children.map((child) => child.completed_at)),
    status: combineStatuses(children.map((child) => child.status)),
    token_count: tokenCount,
  };
}

function sum(nodes: OcrMetricsTreeNode[], key: keyof OcrMetricsTreeNode) {
  return nodes.reduce((total, node) => total + Number(node[key] ?? 0), 0);
}

function minPositive(values: Array<number | undefined>) {
  const positive = values.filter(
    (value): value is number => typeof value === 'number' && value > 0,
  );
  return positive.length ? Math.min(...positive) : 0;
}

function firstValue<K extends keyof OcrMetricsTreeNode>(nodes: OcrMetricsTreeNode[], key: K) {
  return nodes.map((node) => node[key]).find(Boolean) as OcrMetricsTreeNode[K] | undefined;
}

function minText(values: Array<string | undefined>) {
  const present = values.filter((value): value is string => Boolean(value));
  return present.length ? present.sort()[0] : undefined;
}

function maxText(values: Array<string | undefined>) {
  const present = values.filter((value): value is string => Boolean(value));
  return present.length ? present.sort().at(-1) : undefined;
}

function combineStatuses(statuses: string[]) {
  if (statuses.includes('running')) {
    return 'running';
  }
  if (statuses.includes('failed')) {
    return 'failed';
  }
  if (statuses.includes('cancelled')) {
    return 'cancelled';
  }
  if (statuses.includes('completed_with_errors')) {
    return 'completed_with_errors';
  }
  return statuses[0] ?? 'completed';
}

function compareLabels(left: OcrMetricsTreeNode, right: OcrMetricsTreeNode) {
  return left.label.localeCompare(right.label);
}

function comparePageNodes(left: OcrMetricsTreeNode, right: OcrMetricsTreeNode) {
  return (left.page_no ?? 0) - (right.page_no ?? 0);
}
